use crate::core::config::{AppConfig, ConfigStatus, PathResolver};
use crate::infrastructure::db::SettingRepository;
use crate::services::{
    ChatService, HostService, ModelService, ProxyService, RuntimeService,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<RwLock<AppConfig>>,
    pub config_status: Arc<RwLock<ConfigStatus>>,
    pub path_resolver: Arc<PathResolver>,
    pub cli_config_path: Option<String>,
    pub model_service: Arc<ModelService>,
    pub chat_service: Arc<ChatService>,
    pub runtime_service: Arc<RuntimeService>,
    pub host_service: Arc<HostService>,
    pub proxy_service: Arc<ProxyService>,
    pub setting_repo: SettingRepository,
}
