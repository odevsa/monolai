use crate::config::GuiConfig;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running {
        url: String,
        port: u16,
        started_at: Instant,
    },
    Stopping,
    Error(String),
    UnexpectedExit {
        exit_code: Option<i32>,
    },
}

#[derive(Clone)]
pub struct ProcessSupervisor {
    child: Arc<Mutex<Option<Child>>>,
    status: Arc<Mutex<ServerStatus>>,
    intentional_stop: Arc<AtomicBool>,
}

impl ProcessSupervisor {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(ServerStatus::Stopped)),
            intentional_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_status(&self) -> ServerStatus {
        self.status.lock().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        matches!(self.get_status(), ServerStatus::Running { .. })
    }

    /// Locate the `monolai` server binary
    pub fn find_server_binary() -> Result<PathBuf, String> {
        let current_exe = env::current_exe().map_err(|e| e.to_string())?;
        let current_dir = current_exe.parent().unwrap_or_else(|| Path::new("."));

        // 1. Check in the same directory as this GUI binary (installed/packaged mode)
        let local_bin = if cfg!(windows) {
            current_dir.join("monolai.exe")
        } else {
            current_dir.join("monolai")
        };
        if local_bin.exists() {
            return Ok(local_bin);
        }

        // 2. Check development target directories
        let dev_paths = [
            current_dir.join("../../../backend/target/release/monolai"),
            current_dir.join("../../../backend/target/debug/monolai"),
            current_dir.join("../target/release/monolai"),
            current_dir.join("../target/debug/monolai"),
            PathBuf::from("target/release/monolai"),
            PathBuf::from("target/debug/monolai"),
            PathBuf::from("backend/target/release/monolai"),
            PathBuf::from("backend/target/debug/monolai"),
        ];

        for p in &dev_paths {
            if p.exists() {
                return Ok(p.clone());
            }
        }

        // 3. Check system PATH
        if let Ok(path_var) = env::var("PATH") {
            for dir in env::split_paths(&path_var) {
                let p = if cfg!(windows) {
                    dir.join("monolai.exe")
                } else {
                    dir.join("monolai")
                };
                if p.exists() {
                    return Ok(p);
                }
            }
        }

        Err("Could not find 'monolai' server binary in current dir, PATH, or target/".to_string())
    }

    pub fn start(&self, config: &GuiConfig) -> Result<(), String> {
        let binary_path = Self::find_server_binary()?;

        let mut child_guard = self.child.lock().unwrap();
        if child_guard.is_some() {
            return Err("Server is already running".to_string());
        }

        // Pre-flight check: verify if the desired port is already occupied
        {
            let port_test = std::net::TcpListener::bind((config.host.as_str(), config.port))
                .or_else(|_| std::net::TcpListener::bind(("127.0.0.1", config.port)));

            if let Err(e) = port_test {
                let err_msg = format!(
                    "Port {} is already in use ({}). Check if Docker or another instance is running on this port!",
                    config.port, e
                );
                let mut st = self.status.lock().unwrap();
                *st = ServerStatus::Error(err_msg.clone());
                return Err(err_msg);
            }
            // Explicitly drop listener so the port is immediately released before spawn
            drop(port_test);
        }

        self.intentional_stop.store(false, Ordering::SeqCst);
        {
            let mut st = self.status.lock().unwrap();
            *st = ServerStatus::Starting;
        }

        let mut cmd = Command::new(&binary_path);
        if let Some(parent) = binary_path.parent() {
            cmd.current_dir(parent);
        }
        cmd.env("HOST", &config.host);
        cmd.env("PORT", config.port.to_string());
        cmd.env("HARDWARE", &config.hardware);
        cmd.env("MODELS_DIR", &config.models_dir);
        cmd.env("RUNTIMES_DIR", &config.runtimes_dir);
        cmd.arg("--config").arg(GuiConfig::backend_yaml_path());

        // Redirect child stdout and stderr to server.log to avoid invalid handle crashes on Windows GUI
        let log_dir = dirs::data_local_dir()
            .map(|p| p.join("monolai").join("logs"))
            .unwrap_or_else(|| PathBuf::from("logs"));
        let _ = std::fs::create_dir_all(&log_dir);
        let log_file_path = log_dir.join("server.log");

        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file_path)
        {
            if let Ok(file_clone) = file.try_clone() {
                cmd.stdout(file);
                cmd.stderr(file_clone);
            } else {
                cmd.stdout(std::process::Stdio::null());
                cmd.stderr(std::process::Stdio::null());
            }
        } else {
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());
        }

        #[cfg(target_os = "linux")]
        unsafe {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
                Ok(())
            });
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let spawned = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", binary_path.display(), e))?;

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::System::JobObjects::*;

            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if !job.is_null() {
                    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                    SetInformationJobObject(
                        job,
                        JobObjectExtendedLimitInformation,
                        &info as *const _ as *const _,
                        std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                    );
                    AssignProcessToJobObject(job, spawned.as_raw_handle() as _);
                    Box::leak(Box::new(job));
                }
            }
        }

        *child_guard = Some(spawned);
        drop(child_guard);

        // Spawn background monitoring thread
        let child_arc = Arc::clone(&self.child);
        let status_arc = Arc::clone(&self.status);
        let stop_flag = Arc::clone(&self.intentional_stop);
        let port = config.port;
        let url = format!("http://localhost:{}", port);
        let log_file_for_thread = log_file_path.clone();

        thread::spawn(move || {
            let start_time = Instant::now();
            let mut is_up = false;

            // Wait for health endpoint to respond
            for _ in 0..60 {
                // Check if process crashed immediately
                {
                    let mut lock = child_arc.lock().unwrap();
                    if let Some(ref mut child) = *lock {
                        if let Ok(Some(exit_status)) = child.try_wait() {
                            *lock = None;
                            let mut st = status_arc.lock().unwrap();
                            *st = ServerStatus::UnexpectedExit {
                                exit_code: exit_status.code(),
                            };
                            return;
                        }
                    } else {
                        return;
                    }
                }

                // Probe HTTP /health using 127.0.0.1 directly to avoid IPv6 localhost resolution issues on Windows
                let health_url = format!("http://127.0.0.1:{}/health", port);
                let resp = ureq::get(&health_url)
                    .timeout(Duration::from_millis(500))
                    .call()
                    .or_else(|_| {
                        let fallback_url = format!("http://localhost:{}/health", port);
                        ureq::get(&fallback_url)
                            .timeout(Duration::from_millis(500))
                            .call()
                    });

                if let Ok(r) = resp {
                    if r.status() == 200 {
                        is_up = true;
                        break;
                    }
                }

                thread::sleep(Duration::from_millis(300));
            }

            if is_up {
                {
                    let mut st = status_arc.lock().unwrap();
                    *st = ServerStatus::Running {
                        url: url.clone(),
                        port,
                        started_at: start_time,
                    };
                }

                // Continuous health & process monitoring loop
                loop {
                    thread::sleep(Duration::from_millis(1000));

                    if stop_flag.load(Ordering::SeqCst) {
                        break;
                    }

                    let mut lock = child_arc.lock().unwrap();
                    if let Some(ref mut child) = *lock {
                        match child.try_wait() {
                            Ok(Some(exit_status)) => {
                                *lock = None;
                                let was_intentional = stop_flag.load(Ordering::SeqCst);
                                let mut st = status_arc.lock().unwrap();
                                if was_intentional {
                                    *st = ServerStatus::Stopped;
                                } else {
                                    *st = ServerStatus::UnexpectedExit {
                                        exit_code: exit_status.code(),
                                    };
                                }
                                break;
                            }
                            Ok(None) => {
                                // Still running
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            } else {
                // Failed to become healthy in time
                let mut lock = child_arc.lock().unwrap();
                if let Some(ref mut child) = *lock {
                    let _ = child.kill();
                    let _ = child.wait();
                    *lock = None;
                }

                let error_detail = std::fs::read_to_string(&log_file_for_thread)
                    .ok()
                    .and_then(|content| {
                        let trimmed = content.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed.lines().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(" "))
                        }
                    });

                let err_msg = if let Some(detail) = error_detail {
                    format!("Server failed to respond: {}", detail)
                } else {
                    "Server failed to respond on health endpoint within 18s".to_string()
                };

                let mut st = status_arc.lock().unwrap();
                *st = ServerStatus::Error(err_msg);
            }
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        self.intentional_stop.store(true, Ordering::SeqCst);
        {
            let mut st = self.status.lock().unwrap();
            *st = ServerStatus::Stopping;
        }

        let mut child_guard = self.child.lock().unwrap();
        if let Some(mut child) = child_guard.take() {
            #[cfg(unix)]
            {
                let pid = child.id() as i32;
                unsafe {
                    libc::kill(pid, libc::SIGTERM);
                }
            }

            #[cfg(not(unix))]
            {
                let _ = child.kill();
            }

            // Give it up to 2.5 seconds to gracefully exit
            for _ in 0..25 {
                if let Ok(Some(_)) = child.try_wait() {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }

            // Force kill if still lingering
            if let Ok(None) = child.try_wait() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }

        let mut st = self.status.lock().unwrap();
        *st = ServerStatus::Stopped;
        Ok(())
    }

    pub fn restart(&self, config: &GuiConfig) -> Result<(), String> {
        let _ = self.stop();
        thread::sleep(Duration::from_millis(500));
        self.start(config)
    }

    /// Executes a maintenance flag on the backend binary (e.g. `--clean-storage` or `--factory-reset`).
    /// If the server is currently running, stops it first, executes the command, and restarts if it was running.
    pub fn run_maintenance_flag(&self, flag: &str, config: Option<&GuiConfig>) -> Result<String, String> {
        let binary_path = Self::find_server_binary()?;
        let was_running = matches!(self.get_status(), ServerStatus::Running { .. });

        if was_running {
            let _ = self.stop();
            thread::sleep(Duration::from_millis(300));
        }

        let mut cmd = Command::new(&binary_path);
        if let Some(parent) = binary_path.parent() {
            cmd.current_dir(parent);
        }
        cmd.arg(flag);

        if let Some(cfg) = config {
            cmd.arg("--config").arg(GuiConfig::backend_yaml_path());
            cmd.env("HOST", &cfg.host);
            cmd.env("PORT", cfg.port.to_string());
            cmd.env("HARDWARE", &cfg.hardware);
            cmd.env("MODELS_DIR", &cfg.models_dir);
            cmd.env("RUNTIMES_DIR", &cfg.runtimes_dir);
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute {}: {}", binary_path.display(), e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() {
            let err_text = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                format!("Process exited with error code {:?}", output.status.code())
            };
            return Err(err_text);
        }

        if was_running {
            if let Some(cfg) = config {
                let _ = self.start(cfg);
            }
        }

        let last_line = stdout
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("Operation completed successfully.")
            .to_string();

        Ok(last_line)
    }
}

impl Drop for ProcessSupervisor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
