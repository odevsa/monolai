use crate::core::config::{
    get_default_models_dir, get_default_runtimes_dir, is_running_in_docker, load_config,
    save_config, ConfigStatus,
};
use crate::core::error::{AppError, AppResult};
use crate::domain::HardwareReport;
use crate::infrastructure::hardware::detect_hardware;
use crate::state::AppState;
use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SaveConfigRequest {
    /// Absolute or relative path to models directory
    pub models: String,
    /// Absolute or relative path to runtimes directory
    pub runtimes: String,
    /// Optional legacy hardware parameter (deprecated/ignored)
    #[serde(default)]
    #[allow(dead_code)]
    pub hardware: Option<String>,
    /// Optional host address to bind (e.g. 0.0.0.0 or 127.0.0.1)
    #[serde(default)]
    pub host: Option<String>,
    /// Optional port to bind (e.g. 8080)
    #[serde(default)]
    pub port: Option<u16>,
    /// Optional desktop GUI preferences
    #[serde(default)]
    pub gui: Option<GuiPreferencesDto>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GuiPreferencesDto {
    #[serde(default = "default_true")]
    pub minimize_on_start: bool,
    #[serde(default = "default_true")]
    pub autostart_server: bool,
    #[serde(default = "default_true")]
    pub autostart_app: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SaveConfigResponse {
    pub success: bool,
    pub config_status: ConfigStatus,
}

/// Get current configuration status and file paths
#[utoipa::path(
    get,
    path = "/api/config/status",
    tag = "Config",
    responses(
        (status = 200, description = "Current configuration status", body = ConfigStatus)
    )
)]
pub async fn config_status_handler(State(state): State<AppState>) -> Json<ConfigStatus> {
    let (fresh_config, fresh_status) = load_config(state.cli_config_path.as_deref());

    {
        let mut cfg = state.config.write().await;
        *cfg = fresh_config;
    }

    {
        let mut st = state.config_status.write().await;
        *st = fresh_status.clone();
    }

    Json(fresh_status)
}

/// Detect host system hardware acceleration capabilities
#[utoipa::path(
    get,
    path = "/api/hardware/detect",
    tag = "Config",
    responses(
        (status = 200, description = "Detected hardware capabilities and GPUs", body = HardwareReport)
    )
)]
pub async fn hardware_detect_handler() -> Json<HardwareReport> {
    Json(detect_hardware())
}

/// Save setup configuration to config.yaml
#[utoipa::path(
    post,
    path = "/api/config/setup",
    tag = "Config",
    request_body = SaveConfigRequest,
    responses(
        (status = 200, description = "Configuration saved successfully", body = SaveConfigResponse),
        (status = 400, description = "Invalid directory path provided", body = String),
        (status = 500, description = "Failed to save configuration", body = String)
    )
)]
pub async fn save_setup_config_handler(
    State(state): State<AppState>,
    Json(payload): Json<SaveConfigRequest>,
) -> AppResult<Json<SaveConfigResponse>> {
    let in_docker = is_running_in_docker();
    let models = if in_docker {
        get_default_models_dir().to_string_lossy().to_string()
    } else {
        payload.models.trim().to_string()
    };
    let runtimes = if in_docker {
        get_default_runtimes_dir().to_string_lossy().to_string()
    } else {
        payload.runtimes.trim().to_string()
    };
    let host = if in_docker {
        Some("0.0.0.0")
    } else {
        payload.host.as_deref()
    };
    let port = if in_docker {
        std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .or(Some(8080))
    } else {
        payload.port
    };

    if models.trim().is_empty() {
        return Err(AppError::bad_request("Models directory cannot be empty"));
    }
    if runtimes.trim().is_empty() {
        return Err(AppError::bad_request("Runtimes directory cannot be empty"));
    }

    let gui_tuple = payload
        .gui
        .as_ref()
        .map(|g| (g.minimize_on_start, g.autostart_server, g.autostart_app));

    save_config(
        state.cli_config_path.as_deref(),
        &models,
        &runtimes,
        host,
        port,
        gui_tuple,
    )
    .map_err(AppError::internal)?;

    let (fresh_config, fresh_status) = load_config(state.cli_config_path.as_deref());

    {
        let mut cfg = state.config.write().await;
        *cfg = fresh_config;
    }

    {
        let mut st = state.config_status.write().await;
        *st = fresh_status.clone();
    }

    Ok(Json(SaveConfigResponse {
        success: true,
        config_status: fresh_status,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_save_config_request() {
        let json =
            r#"{"models":"/path/models","runtimes":"/path/runtimes","host":"0.0.0.0","port":8080}"#;
        let req: SaveConfigRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.models, "/path/models");
        assert_eq!(req.runtimes, "/path/runtimes");
        assert_eq!(req.host.as_deref(), Some("0.0.0.0"));
        assert_eq!(req.port, Some(8080));
    }
}
