use crate::core::config::AppConfig;
use crate::core::error::{AppError, AppResult};
use crate::domain::{ModelState, RunningModelStatus};
use crate::infrastructure::db::{ModelRepository, SettingRepository};
use crate::infrastructure::downloader::find_installed_binary;
use crate::infrastructure::process::manifests::get_manifest_for_runtime;
use crate::infrastructure::scanner::check_model_file_exists;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

pub struct RunningModel {
    pub model_id: String,
    pub runtime_id: String,
    pub pid: u32,
    pub port: u16,
    pub state: ModelState,
    pub last_active: Instant,
    pub child: Option<Arc<Mutex<Child>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessFileRecord {
    pub model_id: String,
    pub runtime: String,
    pub pid: u32,
    pub port: u16,
}

pub fn read_processes_json(db_dir: &Path) -> HashMap<String, ProcessFileRecord> {
    let file_path = db_dir.join("processes.json");
    if let Ok(content) = std::fs::read_to_string(&file_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

pub fn write_processes_json(db_dir: &Path, records: &HashMap<String, ProcessFileRecord>) {
    let file_path = db_dir.join("processes.json");
    if records.is_empty() {
        let _ = std::fs::remove_file(&file_path);
    } else if let Ok(json) = serde_json::to_string_pretty(records) {
        let _ = std::fs::write(&file_path, json);
    }
}

pub fn save_active_process_to_json(
    db_dir: &Path,
    model_id: &str,
    runtime: &str,
    pid: u32,
    port: u16,
) {
    let mut records = read_processes_json(db_dir);
    records.insert(
        model_id.to_string(),
        ProcessFileRecord {
            model_id: model_id.to_string(),
            runtime: runtime.to_string(),
            pid,
            port,
        },
    );
    write_processes_json(db_dir, &records);
}

pub fn remove_active_process_from_json(db_dir: &Path, model_id: &str) {
    let mut records = read_processes_json(db_dir);
    if records.remove(model_id).is_some() {
        write_processes_json(db_dir, &records);
    }
}

#[derive(Clone)]
pub struct ProcessManager {
    pub running: Arc<Mutex<HashMap<String, RunningModel>>>,
    pub db_dir: PathBuf,
    pub state_broadcast: tokio::sync::broadcast::Sender<Vec<RunningModelStatus>>,
}

impl ProcessManager {
    pub fn new(db_dir: PathBuf) -> Self {
        let (state_broadcast, _) = tokio::sync::broadcast::channel(64);
        Self {
            running: Arc::new(Mutex::new(HashMap::new())),
            db_dir,
            state_broadcast,
        }
    }

    pub async fn get_all_running_status(&self) -> Vec<RunningModelStatus> {
        let map = self.running.lock().await;
        map.values()
            .map(|m| RunningModelStatus {
                model_id: m.model_id.clone(),
                runtime_id: m.runtime_id.clone(),
                pid: m.pid,
                port: m.port,
                state: m.state.clone(),
                idle_seconds: m.last_active.elapsed().as_secs(),
            })
            .collect()
    }

    pub async fn notify_state_changed(&self) {
        let status = self.get_all_running_status().await;
        let _ = self.state_broadcast.send(status);
    }

    pub fn subscribe_state(&self) -> tokio::sync::broadcast::Receiver<Vec<RunningModelStatus>> {
        self.state_broadcast.subscribe()
    }

    pub async fn mark_model_active(&self, model_id: &str) {
        let mut map = self.running.lock().await;
        if let Some(m) = map.get_mut(model_id) {
            m.last_active = Instant::now();
        }
    }
}

pub fn allocate_available_port() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to bind to dynamic port: {}", e))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local port: {}", e))?
        .port();
    drop(listener);
    Ok(port)
}

pub async fn load_model_process(
    pm: &ProcessManager,
    model_repo: &ModelRepository,
    setting_repo: &SettingRepository,
    config: &AppConfig,
    model_id: &str,
) -> AppResult<RunningModelStatus> {
    let model_record = match model_repo.get_by_id(model_id).await? {
        Some(m) => m,
        None => {
            return Err(AppError::not_found(format!(
                "Model '{}' not found in database",
                model_id
            )))
        }
    };

    let models_dir_str = config.models.as_deref().unwrap_or("");
    let models_dir = if models_dir_str.is_empty() {
        crate::core::config::get_default_models_dir()
    } else {
        crate::core::config::expand_tilde(models_dir_str)
    };

    let (file_exists, file_name) = check_model_file_exists(&models_dir, &model_record.flags);
    if !file_exists {
        return Err(AppError::bad_request(format!(
            "Model file '{}' not found on disk. The file may have been moved or deleted.",
            file_name.unwrap_or_else(|| "unknown".to_string())
        )));
    }

    let runtime_id = model_record.runtime.clone();
    let runtimes_dir_str = config.runtimes.as_deref().unwrap_or("");
    let runtimes_dir = crate::core::config::expand_tilde(runtimes_dir_str);

    let manifest = get_manifest_for_runtime(None, &runtime_id).ok_or_else(|| {
        AppError::not_found(format!("Runtime manifest for '{}' not found", runtime_id))
    })?;

    let binary_path = match find_installed_binary(&runtimes_dir, &manifest) {
        Some(path) => path.to_string_lossy().to_string(),
        None => {
            return Err(AppError::bad_request(format!(
                "Runtime '{}' is not installed yet. Please install it before running this model.",
                runtime_id
            )));
        }
    };

    {
        let mut map = pm.running.lock().await;
        if let Some(existing) = map.get_mut(model_id) {
            match &existing.state {
                ModelState::Ready => {
                    existing.last_active = Instant::now();
                    return Ok(RunningModelStatus {
                        model_id: existing.model_id.clone(),
                        runtime_id: existing.runtime_id.clone(),
                        pid: existing.pid,
                        port: existing.port,
                        state: existing.state.clone(),
                        idle_seconds: existing.last_active.elapsed().as_secs(),
                    });
                }
                ModelState::Loading => {
                    return Ok(RunningModelStatus {
                        model_id: existing.model_id.clone(),
                        runtime_id: existing.runtime_id.clone(),
                        pid: existing.pid,
                        port: existing.port,
                        state: existing.state.clone(),
                        idle_seconds: 0,
                    });
                }
                _ => {}
            }
        }

        map.insert(
            model_id.to_string(),
            RunningModel {
                model_id: model_id.to_string(),
                runtime_id: runtime_id.clone(),
                pid: 0,
                port: 0,
                state: ModelState::Loading,
                last_active: Instant::now(),
                child: None,
            },
        );
    }
    pm.notify_state_changed().await;

    let fail_with = |err_msg: String| {
        let pm_clone = pm.clone();
        let id_clone = model_id.to_string();
        let runtime_clone = runtime_id.clone();
        let err_clone = err_msg.clone();
        tokio::spawn(async move {
            {
                let mut map = pm_clone.running.lock().await;
                map.insert(
                    id_clone.clone(),
                    RunningModel {
                        model_id: id_clone,
                        runtime_id: runtime_clone,
                        pid: 0,
                        port: 0,
                        state: ModelState::Error(err_clone),
                        last_active: Instant::now(),
                        child: None,
                    },
                );
            }
            pm_clone.notify_state_changed().await;
        });
        AppError::internal(err_msg)
    };

    let allocated_port = match allocate_available_port() {
        Ok(p) => p,
        Err(e) => return Err(fail_with(e)),
    };

    let parsed_flags: HashMap<String, String> =
        serde_json::from_str(&model_record.flags).unwrap_or_default();

    let mut cmd = Command::new(&binary_path);
    let mut port_injected = false;

    let is_sd_server = runtime_id == "sd-cpp"
        || manifest.binary_name == "sd-server"
        || binary_path.ends_with("sd-server");
    let port_flag = if is_sd_server {
        "--listen-port"
    } else {
        "--port"
    };

    if let Some(parent) = std::path::Path::new(&binary_path).parent() {
        cmd.current_dir(parent);
    }

    for (k, v) in &parsed_flags {
        let val_substituted = v.replace("${PORT}", &allocated_port.to_string());
        if k == "--port" || k == "--listen-port" || k == "-p" {
            port_injected = true;
            cmd.arg(port_flag).arg(&val_substituted);
        } else if v.trim().is_empty() {
            cmd.arg(k);
        } else {
            cmd.arg(k).arg(&val_substituted);
        }
    }

    if !port_injected {
        cmd.arg(port_flag).arg(allocated_port.to_string());
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Err(fail_with(format!(
                "Failed to spawn binary '{}': {}",
                binary_path, e
            )));
        }
    };

    let pid = match child.id() {
        Some(id) => id,
        None => return Err(fail_with("Failed to retrieve child process PID".into())),
    };

    save_active_process_to_json(&pm.db_dir, model_id, &runtime_id, pid, allocated_port);

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let child_ref = Arc::new(Mutex::new(child));

    {
        let mut map = pm.running.lock().await;
        map.insert(
            model_id.to_string(),
            RunningModel {
                model_id: model_id.to_string(),
                runtime_id: runtime_id.clone(),
                pid,
                port: allocated_port,
                state: ModelState::Loading,
                last_active: Instant::now(),
                child: Some(child_ref.clone()),
            },
        );
    }

    let log_ready_signal = Arc::new(tokio::sync::Notify::new());
    let log_ready_signal_clone = log_ready_signal.clone();
    let last_logs = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let last_logs_out = last_logs.clone();
    let last_logs_err = last_logs.clone();

    tokio::spawn(async move {
        if let Some(out) = stdout {
            let mut reader = BufReader::new(out).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                tracing::debug!(target: "upstream_log", "[{}] stdout: {}", pid, line);
                {
                    let mut logs = last_logs_out.lock().unwrap();
                    if logs.len() >= 10 {
                        logs.remove(0);
                    }
                    logs.push(line.clone());
                }
                let lower = line.to_lowercase();
                if lower.contains("all slots are idle and ready to serve requests")
                    || lower.contains("main: model loaded")
                    || lower.contains("server is listening")
                    || lower.contains("listening at")
                    || lower.contains("listening on")
                {
                    log_ready_signal_clone.notify_waiters();
                }
            }
        }
    });

    let log_ready_signal_clone2 = log_ready_signal.clone();
    tokio::spawn(async move {
        if let Some(err) = stderr {
            let mut reader = BufReader::new(err).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                tracing::debug!(target: "upstream_log", "[{}] stderr: {}", pid, line);
                {
                    let mut logs = last_logs_err.lock().unwrap();
                    if logs.len() >= 10 {
                        logs.remove(0);
                    }
                    logs.push(line.clone());
                }
                let lower = line.to_lowercase();
                if lower.contains("all slots are idle and ready to serve requests")
                    || lower.contains("main: model loaded")
                    || lower.contains("server is listening")
                    || lower.contains("listening at")
                    || lower.contains("listening on")
                {
                    log_ready_signal_clone2.notify_waiters();
                }
            }
        }
    });

    let readiness_url = format!("http://127.0.0.1:{}/v1/models", allocated_port);
    let health_url = format!("http://127.0.0.1:{}/health", allocated_port);
    let sdcpp_url = format!("http://127.0.0.1:{}/sdcpp/v1/capabilities", allocated_port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .unwrap_or_default();

    let readiness_timeout_secs: u64 = setting_repo.get_parsed("readiness_timeout", 120).await;
    let start_time = Instant::now();
    let mut is_ready = false;

    while start_time.elapsed() < Duration::from_secs(readiness_timeout_secs) {
        {
            let mut child_guard = child_ref.lock().await;
            if let Ok(Some(status)) = child_guard.try_wait() {
                tokio::time::sleep(Duration::from_millis(150)).await;
                let logs = last_logs.lock().unwrap();
                let detail = if logs.is_empty() {
                    String::new()
                } else {
                    format!(": {}", logs.join(" | "))
                };
                return Err(fail_with(format!(
                    "Child process exited prematurely with status: {}{}",
                    status, detail
                )));
            }
        }

        let mut http_ready = false;
        if let Ok(resp) = client.get(&readiness_url).send().await {
            if resp.status().is_success() {
                http_ready = true;
            }
        } else if let Ok(resp) = client.get(&health_url).send().await {
            if resp.status().is_success() {
                http_ready = true;
            }
        } else if let Ok(resp) = client.get(&sdcpp_url).send().await {
            if resp.status().is_success() {
                http_ready = true;
            }
        }

        if http_ready {
            is_ready = true;
            break;
        }

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(500)) => {},
            _ = log_ready_signal.notified() => {
                is_ready = true;
                break;
            }
        }
    }

    if is_ready {
        let status = {
            let mut map = pm.running.lock().await;
            if let Some(m) = map.get_mut(model_id) {
                m.state = ModelState::Ready;
                m.last_active = Instant::now();
                Some(RunningModelStatus {
                    model_id: m.model_id.clone(),
                    runtime_id: m.runtime_id.clone(),
                    pid: m.pid,
                    port: m.port,
                    state: ModelState::Ready,
                    idle_seconds: 0,
                })
            } else {
                None
            }
        };

        if let Some(s) = status {
            pm.notify_state_changed().await;
            return Ok(s);
        }
    }

    let _ = unload_model_process(pm, setting_repo, model_id).await;
    Err(fail_with(format!(
        "Readiness check timed out after {}s for model '{}'",
        readiness_timeout_secs, model_id
    )))
}

pub async fn unload_model_process(
    pm: &ProcessManager,
    setting_repo: &SettingRepository,
    model_id: &str,
) -> AppResult<()> {
    remove_active_process_from_json(&pm.db_dir, model_id);

    let target_child = {
        let mut map = pm.running.lock().await;
        if let Some(running) = map.remove(model_id) {
            running
        } else {
            return Ok(());
        }
    };
    pm.notify_state_changed().await;

    let unload_timeout_secs: u64 = setting_repo.get_parsed("unload_timeout", 10).await;

    if target_child.pid > 0 {
        #[cfg(unix)]
        unsafe {
            libc::kill(target_child.pid as i32, libc::SIGTERM);
        }
    }

    if let Some(child_arc) = target_child.child {
        let start = Instant::now();
        loop {
            {
                let mut child_guard = child_arc.lock().await;
                if let Ok(Some(_)) = child_guard.try_wait() {
                    break;
                }
            }
            if start.elapsed() >= Duration::from_secs(unload_timeout_secs) {
                tracing::warn!(
                    "Model '{}' (pid: {}) did not exit after {}s SIGTERM. Sending SIGKILL...",
                    model_id,
                    target_child.pid,
                    unload_timeout_secs
                );

                if target_child.pid > 0 {
                    #[cfg(unix)]
                    unsafe {
                        libc::kill(target_child.pid as i32, libc::SIGKILL);
                    }
                }

                let mut child_guard = child_arc.lock().await;
                let _ = child_guard.kill().await;
                let _ = child_guard.wait().await;
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    Ok(())
}

pub async fn unload_all_model_processes(
    pm: &ProcessManager,
    setting_repo: &SettingRepository,
) -> AppResult<()> {
    let running_ids: Vec<String> = {
        let map = pm.running.lock().await;
        map.keys().cloned().collect()
    };

    for id in running_ids {
        let _ = unload_model_process(pm, setting_repo, &id).await;
    }

    Ok(())
}

pub fn start_idle_auto_unload_loop(pm: ProcessManager, setting_repo: SettingRepository) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            let enabled: bool = setting_repo
                .get_parsed("idle_auto_unload_enabled", true)
                .await;
            if !enabled {
                continue;
            }

            let timeout_secs: u64 = setting_repo.get_parsed("idle_timeout_seconds", 300).await;

            let expired_models: Vec<String> = {
                let map = pm.running.lock().await;
                map.values()
                    .filter(|m| {
                        m.state == ModelState::Ready
                            && m.last_active.elapsed().as_secs() >= timeout_secs
                    })
                    .map(|m| m.model_id.clone())
                    .collect()
            };

            for id in expired_models {
                tracing::info!(
                    "Auto-unloading model '{}' due to {}s of inactivity",
                    id,
                    timeout_secs
                );
                let _ = unload_model_process(&pm, &setting_repo, &id).await;
            }
        }
    });
}

pub async fn adopt_or_clean_orphans(pm: &ProcessManager) {
    let records = read_processes_json(&pm.db_dir);
    let mut updated_records = records.clone();
    let mut changed = false;
    let mut state_changed = false;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap_or_default();

    for (model_id, record) in &records {
        let pid = record.pid;
        let port = record.port;
        let runtime = &record.runtime;

        #[cfg(unix)]
        let is_alive_os = if pid > 0 {
            unsafe { libc::kill(pid as i32, 0) == 0 }
        } else {
            false
        };

        #[cfg(not(unix))]
        let is_alive_os = pid > 0;

        let mut is_responding_http = false;
        if is_alive_os && port > 0 {
            let probe_url = format!("http://127.0.0.1:{}/health", port);
            if let Ok(resp) = client.get(&probe_url).send().await {
                if resp.status().is_success() {
                    is_responding_http = true;
                }
            } else {
                let probe_url2 = format!("http://127.0.0.1:{}/v1/models", port);
                if let Ok(resp) = client.get(&probe_url2).send().await {
                    if resp.status().is_success() {
                        is_responding_http = true;
                    }
                }
            }
        }

        if is_alive_os && is_responding_http {
            tracing::info!(
                "Adopting active orphan process for model '{}' (pid: {}, port: {}, runtime: {}) from processes.json",
                model_id,
                pid,
                port,
                runtime
            );
            let mut map = pm.running.lock().await;
            map.insert(
                model_id.clone(),
                RunningModel {
                    model_id: model_id.clone(),
                    runtime_id: runtime.clone(),
                    pid,
                    port,
                    state: ModelState::Ready,
                    last_active: Instant::now(),
                    child: None,
                },
            );
            drop(map);
            state_changed = true;
        } else {
            tracing::info!(
                "Cleaning up dead/unresponsive orphan process record for model '{}' (pid: {}) from processes.json",
                model_id,
                pid
            );
            if is_alive_os && pid > 0 {
                #[cfg(unix)]
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
            updated_records.remove(model_id);
            changed = true;
        }
    }

    if changed {
        write_processes_json(&pm.db_dir, &updated_records);
    }
    if state_changed {
        pm.notify_state_changed().await;
    }
}

pub async fn swap_model_process(
    pm: &ProcessManager,
    model_repo: &ModelRepository,
    setting_repo: &SettingRepository,
    config: &AppConfig,
    model_id: &str,
) -> AppResult<RunningModelStatus> {
    if model_repo.get_by_id(model_id).await?.is_none() {
        return Err(AppError::not_found(format!(
            "Model '{}' not found in database",
            model_id
        )));
    }

    let wait_start = Instant::now();
    let max_loading_wait = Duration::from_secs(120);

    loop {
        let (_is_ready, is_loading, is_error) = {
            let mut map = pm.running.lock().await;
            if let Some(existing) = map.get_mut(model_id) {
                match &existing.state {
                    ModelState::Ready => {
                        existing.last_active = Instant::now();
                        return Ok(RunningModelStatus {
                            model_id: existing.model_id.clone(),
                            runtime_id: existing.runtime_id.clone(),
                            pid: existing.pid,
                            port: existing.port,
                            state: existing.state.clone(),
                            idle_seconds: 0,
                        });
                    }
                    ModelState::Loading => (false, true, None),
                    ModelState::Error(err) => (false, false, Some(err.clone())),
                    _ => (false, false, None),
                }
            } else {
                (false, false, None)
            }
        };

        if let Some(err) = is_error {
            return Err(AppError::internal(format!("Model failed to load: {}", err)));
        }

        if is_loading {
            if wait_start.elapsed() >= max_loading_wait {
                return Err(AppError::internal(format!(
                    "Timed out waiting for model '{}' to finish loading",
                    model_id
                )));
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
            continue;
        }

        break;
    }

    unload_all_model_processes(pm, setting_repo).await?;
    load_model_process(pm, model_repo, setting_repo, config, model_id).await
}
