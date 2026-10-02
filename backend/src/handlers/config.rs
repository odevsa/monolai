use crate::config::{load_config, save_config, ConfigStatus};
use crate::runtimes::hardware::{detect_hardware, HardwareReport};
use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SaveConfigRequest {
    /// Absolute or relative path to models directory
    pub models: String,
    /// Absolute or relative path to runtimes directory
    pub runtimes: String,
    /// Target hardware acceleration: "auto", "cpu", "cuda", "rocm", or "vulkan"
    #[serde(default = "default_hardware")]
    pub hardware: String,
}

fn default_hardware() -> String {
    "auto".to_string()
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
        let mut cfg = state.config.lock().unwrap();
        *cfg = fresh_config;
    }

    {
        let mut st = state.config_status.lock().unwrap();
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
) -> Result<Json<SaveConfigResponse>, (StatusCode, String)> {
    if payload.models.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Models directory cannot be empty".to_string()));
    }
    if payload.runtimes.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Runtimes directory cannot be empty".to_string()));
    }

    // Save configuration file to disk
    if let Err(err) = save_config(
        state.cli_config_path.as_deref(),
        payload.models.trim(),
        payload.runtimes.trim(),
        payload.hardware.trim(),
    ) {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err));
    }

    // Reload configuration in memory
    let (fresh_config, fresh_status) = load_config(state.cli_config_path.as_deref());

    {
        let mut cfg = state.config.lock().unwrap();
        *cfg = fresh_config;
    }

    {
        let mut st = state.config_status.lock().unwrap();
        *st = fresh_status.clone();
    }

    Ok(Json(SaveConfigResponse {
        success: true,
        config_status: fresh_status,
    }))
}
