pub mod binary_locator;
pub mod health;
pub mod process;
pub mod worker;

use std::sync::mpsc::{channel, Receiver, Sender};
use eframe::egui;

use crate::core::config::GuiConfig;
use crate::core::events::{SupervisorEvent, UiCommand};
use worker::SupervisorWorker;

/// Handle owned by the GUI thread to send commands and receive events from the supervisor.
pub struct SupervisorHandle {
    command_tx: Sender<UiCommand>,
    event_rx: Receiver<SupervisorEvent>,
}

impl SupervisorHandle {
    pub fn new(egui_ctx: egui::Context) -> Self {
        let (command_tx, command_rx) = channel();
        let (event_tx, event_rx) = channel();

        SupervisorWorker::spawn(command_rx, event_tx, egui_ctx);

        Self {
            command_tx,
            event_rx,
        }
    }

    pub fn start(&self, config: GuiConfig) {
        let _ = self.command_tx.send(UiCommand::StartServer(config));
    }

    pub fn stop(&self) {
        let _ = self.command_tx.send(UiCommand::StopServer);
    }

    pub fn restart(&self, config: GuiConfig) {
        let _ = self.command_tx.send(UiCommand::RestartServer(config));
    }

    pub fn run_maintenance(&self, flag: &str, config: Option<GuiConfig>) {
        let _ = self.command_tx.send(UiCommand::RunMaintenance {
            flag: flag.to_string(),
            config,
        });
    }

    pub fn notify_config(&self, config: GuiConfig) {
        let _ = self.command_tx.send(UiCommand::NotifyBackendConfig(config));
    }

    pub fn shutdown(&self) {
        let _ = self.command_tx.send(UiCommand::Shutdown);
    }

    pub fn poll_event(&self) -> Option<SupervisorEvent> {
        self.event_rx.try_recv().ok()
    }
}
