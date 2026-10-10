use std::path::PathBuf;
use std::process::Child;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;

use crate::core::config::GuiConfig;
use crate::core::events::{SupervisorEvent, UiCommand};
use crate::core::status::ServerStatus;
use crate::supervisor::{binary_locator, health, process};

pub struct SupervisorWorker {
    command_rx: Receiver<UiCommand>,
    event_tx: Sender<SupervisorEvent>,
    egui_ctx: egui::Context,
    child: Option<Child>,
    log_path: Option<PathBuf>,
    status: ServerStatus,
    active_config: Option<GuiConfig>,
}

impl SupervisorWorker {
    pub fn spawn(
        command_rx: Receiver<UiCommand>,
        event_tx: Sender<SupervisorEvent>,
        egui_ctx: egui::Context,
    ) -> thread::JoinHandle<()> {
        thread::Builder::new()
            .name("monolai-supervisor".to_string())
            .spawn(move || {
                let mut worker = Self {
                    command_rx,
                    event_tx,
                    egui_ctx,
                    child: None,
                    log_path: None,
                    status: ServerStatus::Stopped,
                    active_config: None,
                };
                worker.run();
            })
            .expect("Failed to spawn supervisor worker thread")
    }

    fn emit_status(&mut self, new_status: ServerStatus) {
        self.status = new_status.clone();
        let _ = self.event_tx.send(SupervisorEvent::StatusChanged(new_status));
        self.egui_ctx.request_repaint();
    }

    fn emit_event(&self, event: SupervisorEvent) {
        let _ = self.event_tx.send(event);
        self.egui_ctx.request_repaint();
    }

    fn run(&mut self) {
        loop {
            let timeout = if matches!(self.status, ServerStatus::Running { .. }) {
                Duration::from_millis(1000)
            } else {
                Duration::from_secs(3600) // Sleep until next command when idle
            };

            match self.command_rx.recv_timeout(timeout) {
                Ok(UiCommand::StartServer(config)) => {
                    self.handle_start(config);
                }
                Ok(UiCommand::StopServer) => {
                    self.handle_stop();
                }
                Ok(UiCommand::RestartServer(config)) => {
                    self.handle_stop();
                    thread::sleep(Duration::from_millis(300));
                    self.handle_start(config);
                }
                Ok(UiCommand::RunMaintenance { flag, config }) => {
                    self.handle_maintenance(&flag, config.as_ref());
                }
                Ok(UiCommand::NotifyBackendConfig(config)) => {
                    self.handle_notify_config(config);
                }
                Ok(UiCommand::Shutdown) => {
                    self.handle_stop();
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    self.check_child_liveness();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // GUI thread closed
                    self.handle_stop();
                    break;
                }
            }
        }
    }

    fn handle_start(&mut self, config: GuiConfig) {
        if self.child.is_some() {
            return;
        }

        // 1. Port conflict pre-flight check
        if let Err(e) = health::check_port_available(&config.host, config.port) {
            self.emit_status(ServerStatus::Error(e.clone()));
            self.emit_event(SupervisorEvent::AlertMessage(e));
            return;
        }

        // 2. Binary resolution
        let binary_path = match binary_locator::find_server_binary() {
            Ok(p) => p,
            Err(e) => {
                self.emit_status(ServerStatus::Error(e.clone()));
                self.emit_event(SupervisorEvent::AlertMessage(e));
                return;
            }
        };

        self.emit_status(ServerStatus::Starting);

        // 3. Spawning process
        match process::spawn_server_process(&binary_path, &config) {
            Ok((child_proc, log_file)) => {
                self.child = Some(child_proc);
                self.log_path = Some(log_file);
                self.active_config = Some(config.clone());
            }
            Err(e) => {
                self.emit_status(ServerStatus::Error(e.clone()));
                self.emit_event(SupervisorEvent::AlertMessage(e));
                return;
            }
        }

        // 4. Polling health probe
        let start_time = Instant::now();
        let port = config.port;
        let url = format!("http://localhost:{}", port);
        let mut is_up = false;

        for _ in 0..60 {
            // Check if process crashed immediately during probe
            if let Some(ref mut child) = self.child {
                if let Ok(Some(exit_status)) = child.try_wait() {
                    let code = exit_status.code();
                    self.child = None;
                    self.emit_status(ServerStatus::UnexpectedExit { exit_code: code });
                    return;
                }
            }

            if health::probe_health(port, Duration::from_millis(500)) {
                is_up = true;
                break;
            }

            thread::sleep(Duration::from_millis(300));
        }

        if is_up {
            self.emit_status(ServerStatus::Running {
                url,
                port,
                started_at: start_time,
            });
        } else {
            // Failed to become healthy in time
            if let Some(mut child) = self.child.take() {
                let _ = process::stop_server_process(&mut child);
            }

            let log_detail = self
                .log_path
                .as_ref()
                .and_then(|p| process::read_recent_log_errors(p));

            let err_msg = if let Some(detail) = log_detail {
                format!("Server failed to respond: {}", detail)
            } else {
                "Server failed to respond on health endpoint within 18s".to_string()
            };

            self.emit_status(ServerStatus::Error(err_msg.clone()));
            self.emit_event(SupervisorEvent::AlertMessage(err_msg));
        }
    }

    fn handle_stop(&mut self) {
        if self.child.is_some() {
            self.emit_status(ServerStatus::Stopping);
            if let Some(mut child) = self.child.take() {
                let _ = process::stop_server_process(&mut child);
            }
        }
        self.emit_status(ServerStatus::Stopped);
    }

    fn check_child_liveness(&mut self) {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(exit_status)) => {
                    let code = exit_status.code();
                    self.child = None;
                    self.emit_status(ServerStatus::UnexpectedExit { exit_code: code });
                }
                Ok(None) => {
                    // Still running healthily
                }
                Err(_) => {
                    self.child = None;
                    self.emit_status(ServerStatus::UnexpectedExit { exit_code: None });
                }
            }
        }
    }

    fn handle_maintenance(&mut self, flag: &str, config: Option<&GuiConfig>) {
        let was_running = matches!(self.status, ServerStatus::Running { .. });
        let restore_config = self.active_config.clone().or_else(|| config.cloned());

        if was_running {
            self.handle_stop();
            thread::sleep(Duration::from_millis(300));
        }

        let binary_path = match binary_locator::find_server_binary() {
            Ok(p) => p,
            Err(e) => {
                self.emit_event(SupervisorEvent::MaintenanceResult {
                    flag: flag.to_string(),
                    result: Err(e),
                });
                return;
            }
        };

        let result = process::run_maintenance_command(&binary_path, flag, config);

        self.emit_event(SupervisorEvent::MaintenanceResult {
            flag: flag.to_string(),
            result,
        });

        if was_running {
            if let Some(cfg) = restore_config {
                self.handle_start(cfg);
            }
        }
    }

    fn handle_notify_config(&mut self, config: GuiConfig) {
        let result = if let ServerStatus::Running { port, .. } = self.status {
            health::notify_backend_config(port, &config)
        } else {
            Ok(())
        };

        self.emit_event(SupervisorEvent::ConfigNotificationResult(result));
    }
}
