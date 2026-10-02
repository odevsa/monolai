use crate::config::{AppConfig, ConfigStatus};
use crate::runtimes::installer::RuntimeInstallerManager;
use crate::runtimes::process_manager::ProcessManager;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use sysinfo::System;

#[derive(Clone)]
pub struct AppState {
    pub sys: Arc<Mutex<System>>,
    pub config: Arc<Mutex<AppConfig>>,
    pub config_status: Arc<Mutex<ConfigStatus>>,
    pub cli_config_path: Option<String>,
    pub db: SqlitePool,
    pub process_manager: ProcessManager,
    pub installer_manager: Arc<RuntimeInstallerManager>,
}
