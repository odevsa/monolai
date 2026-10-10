use crate::core::config::GuiConfig;
use crate::core::status::ServerStatus;

/// Commands sent from the UI (or Tray) thread to the background Supervisor worker.
#[derive(Debug)]
pub enum UiCommand {
    StartServer(GuiConfig),
    StopServer,
    RestartServer(GuiConfig),
    RunMaintenance {
        flag: String,
        config: Option<GuiConfig>,
    },
    NotifyBackendConfig(GuiConfig),
    Shutdown,
}

/// Events emitted from the background Supervisor worker to the UI thread.
#[derive(Debug, Clone)]
pub enum SupervisorEvent {
    StatusChanged(ServerStatus),
    MaintenanceResult {
        flag: String,
        result: Result<String, String>,
    },
    ConfigNotificationResult(Result<(), String>),
    AlertMessage(String),
}

/// Actions initiated from the system tray menu.
#[derive(Debug, Clone)]
pub enum TrayAction {
    OpenDashboard,
    Quit,
}
