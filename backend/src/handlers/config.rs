use crate::config::{
    get_default_models_dir, get_default_runtimes_dir, is_running_in_docker, load_config,
    save_config, ConfigStatus,
};
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
    /// Optional host address to bind (e.g. 0.0.0.0 or 127.0.0.1)
    pub host: Option<String>,
    /// Optional port to bind (e.g. 8080)
    pub port: Option<u16>,
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
        std::env::var("PORT").ok().and_then(|p| p.parse().ok()).or(Some(8080))
    } else {
        payload.port
    };

    if models.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Models directory cannot be empty".to_string()));
    }
    if runtimes.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Runtimes directory cannot be empty".to_string()));
    }

    // Save configuration file to disk
    if let Err(err) = save_config(
        state.cli_config_path.as_deref(),
        &models,
        &runtimes,
        payload.hardware.trim(),
        host,
        port,
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
