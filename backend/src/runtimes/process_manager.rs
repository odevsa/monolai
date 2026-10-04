use crate::config::AppConfig;
use crate::db::models::get_model_by_id;
use crate::runtimes::manifest_loader::get_manifest_for_runtime;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(tag = "status", content = "message")]
pub enum ModelState {
    Idle,
    Loading,
    Ready,
    Error(String),
}

pub struct RunningModel {
    pub model_id: String,
    pub runtime_id: String,
    pub pid: u32,
    pub port: u16,
    pub state: ModelState,
    pub last_active: Instant,
    pub child: Option<Arc<Mutex<Child>>>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RunningModelStatus {
    pub model_id: String,
    pub runtime_id: String,
    pub pid: u32,
    pub port: u16,
    pub state: ModelState,
    pub idle_seconds: u64,
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

pub fn save_active_process_to_json(db_dir: &Path, model_id: &str, runtime: &str, pid: u32, port: u16) {
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
}

impl ProcessManager {
    pub fn new(db_dir: PathBuf) -> Self {
        Self {
            running: Arc::new(Mutex::new(HashMap::new())),
            db_dir,
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

pub async fn get_all_running_status(pm: &ProcessManager) -> Vec<RunningModelStatus> {
    let map = pm.running.lock().await;
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

#[allow(dead_code)]
pub async fn mark_model_active(pm: &ProcessManager, model_id: &str) {
    let mut map = pm.running.lock().await;
    if let Some(m) = map.get_mut(model_id) {
        m.last_active = Instant::now();
    }
}

pub async fn load_model_process(
    pm: &ProcessManager,
    db: &SqlitePool,
    config: &AppConfig,
    model_id: &str,
) -> Result<RunningModelStatus, String> {
    // 1. Query model record from SQLite DB first (if non-existent, do nothing)
    let model_record = match get_model_by_id(db, model_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return Err(format!("Model '{}' not found in database", model_id)),
        Err(e) => return Err(format!("Database query error: {}", e)),
    };

    let models_dir_str = config.models.as_deref().unwrap_or("");
    let models_dir = if models_dir_str.is_empty() {
        crate::config::get_default_models_dir()
    } else {
        crate::config::expand_tilde(models_dir_str)
    };
    let (file_exists, file_name) = crate::runtimes::model_scanner::check_model_file_exists(&models_dir, &model_record.flags);
    if !file_exists {
        return Err(format!(
            "Model file '{}' not found on disk. The file may have been moved or deleted.",
            file_name.unwrap_or_else(|| "unknown".to_string())
        ));
    }

    let runtime_id = model_record.runtime.clone();

    // 2. Resolve installed binary path from directory
    let runtimes_dir_str = config.runtimes.as_deref().unwrap_or("");
    let runtimes_dir = crate::config::expand_tilde(runtimes_dir_str);

    let manifest = get_manifest_for_runtime(&runtime_id).ok_or_else(|| {
        format!("Runtime manifest for '{}' not found", runtime_id)
    })?;

    let binary_path = match crate::runtimes::installer::find_installed_binary(&runtimes_dir, &manifest) {
        Some(path) => path.to_string_lossy().to_string(),
        None => {
            return Err(format!(
                "Runtime '{}' is not installed yet. Please install it from the Runtimes store before running this model.",
                runtime_id
            ));
        }
    };

    // 3. Check current running state
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

        // Insert initial loading state
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

    // Helper to cleanup and set error state on process launch/ready failure
    let fail_with = |err_msg: String| {
        let pm_clone = pm.clone();
        let id_clone = model_id.to_string();
        let runtime_clone = runtime_id.clone();
        let err_clone = err_msg.clone();
        tokio::spawn(async move {
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
        });
        err_msg
    };

    // 4. Allocate dynamic port
    let allocated_port = match allocate_available_port() {
        Ok(p) => p,
        Err(e) => return Err(fail_with(e)),
    };


    // 5. Parse flags & build CLI command
    let parsed_flags: HashMap<String, String> = serde_json::from_str(&model_record.flags)
        .unwrap_or_default();

    let mut cmd = Command::new(&binary_path);
    let mut port_injected = false;

    for (k, v) in &parsed_flags {
        let val_substituted = v.replace("${PORT}", &allocated_port.to_string());
        if k == "--port" || k == "-p" {
            port_injected = true;
            cmd.arg(k).arg(&val_substituted);
        } else if v.trim().is_empty() {
            cmd.arg(k);
        } else {
            cmd.arg(k).arg(&val_substituted);
        }
    }

    if !port_injected {
        cmd.arg("--port").arg(allocated_port.to_string());
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    // 6. Spawn child process
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

    // Record process entry in processes.json for orphan tracking
    save_active_process_to_json(&pm.db_dir, model_id, &runtime_id, pid, allocated_port);

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let child_ref = Arc::new(Mutex::new(child));

    // Update in-memory state
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

    // Monitor stdout/stderr for log lines & readiness signal
    let log_ready_signal = Arc::new(tokio::sync::Notify::new());
    let log_ready_signal_clone = log_ready_signal.clone();

    tokio::spawn(async move {
        if let Some(out) = stdout {
            let mut reader = BufReader::new(out).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                tracing::debug!(target: "upstream_log", "[{}] stdout: {}", pid, line);
                let lower = line.to_lowercase();
                if lower.contains("server listening on")
                    || lower.contains("http server listening")
                    || lower.contains("ready")
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
                let lower = line.to_lowercase();
                if lower.contains("server listening on")
                    || lower.contains("http server listening")
                    || lower.contains("ready")
                {
                    log_ready_signal_clone2.notify_waiters();
                }
            }
        }
    });

    // 7. Perform readiness check (HTTP polling + log signal fallback)
    let ready_path = "/v1/models";

    let readiness_url = format!("http://127.0.0.1:{}{}", allocated_port, ready_path);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .unwrap_or_default();

    // Read readiness timeout from settings DB table, default 120s
    let readiness_timeout_secs: u64 = match sqlx::query_scalar::<_, String>(
        "SELECT value FROM settings WHERE key = 'readiness_timeout'",
    )
    .fetch_optional(db)
    .await
    {
        Ok(Some(val)) => val.parse().unwrap_or(120),
        _ => 120,
    };

    let start_time = Instant::now();
    let mut is_ready = false;

    while start_time.elapsed() < Duration::from_secs(readiness_timeout_secs) {
        // Check if child process exited prematurely
        {
            let mut child_guard = child_ref.lock().await;
            if let Ok(Some(status)) = child_guard.try_wait() {
                return Err(fail_with(format!(
                    "Child process exited prematurely with status: {}",
                    status
                )));
            }
        }

        // Check HTTP readiness endpoint
        if let Ok(resp) = client.get(&readiness_url).send().await {
            if resp.status().is_success() {
                is_ready = true;
                break;
            }
        }

        // Wait 500ms or log signal notification
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(500)) => {},
            _ = log_ready_signal.notified() => {
                is_ready = true;
                break;
            }
        }
    }

    if is_ready {
        let mut map = pm.running.lock().await;
        if let Some(m) = map.get_mut(model_id) {
            m.state = ModelState::Ready;
            m.last_active = Instant::now();
            return Ok(RunningModelStatus {
                model_id: m.model_id.clone(),
                runtime_id: m.runtime_id.clone(),
                pid: m.pid,
                port: m.port,
                state: ModelState::Ready,
                idle_seconds: 0,
            });
        }
    }

    // Timed out: kill process & return error
    let _ = unload_model_process(pm, db, model_id).await;
    Err(fail_with(format!(
        "Readiness check timed out after {}s for model '{}'",
        readiness_timeout_secs, model_id
    )))
}

pub async fn unload_model_process(
    pm: &ProcessManager,
    db: &SqlitePool,
    model_id: &str,
) -> Result<(), String> {
    // Clean up record from processes.json
    remove_active_process_from_json(&pm.db_dir, model_id);

    let target_child = {
        let mut map = pm.running.lock().await;
        if let Some(running) = map.remove(model_id) {
            running
        } else {
            return Ok(());
        }
    };

    // Read unload timeout from settings DB, default 10s
    let unload_timeout_secs: u64 = match sqlx::query_scalar::<_, String>(
        "SELECT value FROM settings WHERE key = 'unload_timeout'",
    )
    .fetch_optional(db)
    .await
    {
        Ok(Some(val)) => val.parse().unwrap_or(10),
        _ => 10,
    };

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

pub async fn unload_all_model_processes(pm: &ProcessManager, db: &SqlitePool) -> Result<(), String> {
    let running_ids: Vec<String> = {
        let map = pm.running.lock().await;
        map.keys().cloned().collect()
    };

    for id in running_ids {
        let _ = unload_model_process(pm, db, &id).await;
    }

    Ok(())
}

pub fn start_idle_auto_unload_loop(pm: ProcessManager, db: SqlitePool) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            let enabled: bool = match sqlx::query_scalar::<_, String>(
                "SELECT value FROM settings WHERE key = 'idle_auto_unload_enabled'",
            )
            .fetch_optional(&db)
            .await
            {
                Ok(Some(val)) => val.parse().unwrap_or(true),
                _ => true,
            };

            if !enabled {
                continue;
            }

            let timeout_secs: u64 = match sqlx::query_scalar::<_, String>(
                "SELECT value FROM settings WHERE key = 'idle_timeout_seconds'",
            )
            .fetch_optional(&db)
            .await
            {
                Ok(Some(val)) => val.parse().unwrap_or(300),
                _ => 300,
            };

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
                let _ = unload_model_process(&pm, &db, &id).await;
            }
        }
    });
}

pub async fn adopt_or_clean_orphans(pm: &ProcessManager) {
    let records = read_processes_json(&pm.db_dir);
    let mut updated_records = records.clone();
    let mut changed = false;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap_or_default();

    for (model_id, record) in &records {
        let pid = record.pid;
        let port = record.port;
        let runtime = &record.runtime;

        let mut is_alive_os = false;
        if pid > 0 {
            #[cfg(unix)]
            unsafe {
                if libc::kill(pid as i32, 0) == 0 {
                    is_alive_os = true;
                }
            }
        }

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
}

pub async fn swap_model_process(
    pm: &ProcessManager,
    db: &SqlitePool,
    config: &AppConfig,
    model_id: &str,
) -> Result<RunningModelStatus, String> {
    // 1. Verify model exists in DB first
    if get_model_by_id(db, model_id).await.ok().flatten().is_none() {
        return Err(format!("Model '{}' not found in database", model_id));
    }

    // 2. Check if target model is already running and ready/loading
    {
        let mut map = pm.running.lock().await;
        if let Some(existing) = map.get_mut(model_id) {
            if matches!(existing.state, ModelState::Ready | ModelState::Loading) {
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
        }
    }

    // 3. Unload all running model processes if a different model is active
    unload_all_model_processes(pm, db).await?;

    // 4. Load target model
    load_model_process(pm, db, config, model_id).await
}




