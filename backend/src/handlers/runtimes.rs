use crate::runtimes::hardware::resolve_target_acceleration;
use crate::runtimes::installer::{
    find_installed_binary, install_runtime, uninstall_runtime, InstallProgress, RuntimeItem,
};
use crate::runtimes::manifest_loader::get_runtime_manifests;
use crate::state::AppState;
use async_stream::stream;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        Json,
    },
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct InstallRuntimeRequest {
    /// Optional hardware acceleration override ("auto", "cpu", "cuda", "rocm", "vulkan")
    pub hardware: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OperationResponse {
    pub success: bool,
    pub message: String,
}

/// List all available runtimes and their installation status
#[utoipa::path(
    get,
    path = "/api/runtimes",
    tag = "Runtimes",
    responses(
        (status = 200, description = "List of all supported runtimes", body = Vec<RuntimeItem>)
    )
)]
pub async fn runtimes_handler(State(state): State<AppState>) -> Json<Vec<RuntimeItem>> {
    let (runtimes_dir, configured_hardware) = {
        let config = state.config.lock().unwrap();
        let dir = config.runtimes.as_deref().unwrap_or("").to_string();
        let hw = config.hardware.clone();
        (crate::config::expand_tilde(dir), hw)
    };

    let hw_report = crate::runtimes::hardware::detect_hardware();
    let active_accel = resolve_target_acceleration(configured_hardware.as_deref());
    let manifests = get_runtime_manifests();
    let mut items = Vec::new();

    for manifest in manifests {
        let binary_path = find_installed_binary(&runtimes_dir, &manifest);
        let is_installed = binary_path.is_some();
        let installed_path = binary_path.map(|p| p.to_string_lossy().to_string());
        let progress = state.installer_manager.get_progress(&manifest.id);
        let installed_accel = crate::runtimes::installer::get_installed_acceleration(&runtimes_dir, &manifest.id);

        let avail_keys = manifest.get_available_accelerations(&hw_report.os, &hw_report.arch);
        let mut avail_options = Vec::new();
        let mut recommended_found = false;

        for key in avail_keys {
            let is_rec = if !recommended_found {
                if key == active_accel
                    || (active_accel == "cuda" && key.starts_with("cuda"))
                    || (active_accel == "metal" && key == "metal")
                {
                    recommended_found = true;
                    true
                } else {
                    false
                }
            } else {
                false
            };

            let label = crate::runtimes::manifest_loader::format_acceleration_label(&key);
            avail_options.push(crate::runtimes::manifest_loader::AccelerationOption {
                id: key,
                label,
                is_recommended: is_rec,
            });
        }

        if !recommended_found && !avail_options.is_empty() {
            avail_options[0].is_recommended = true;
        }

        items.push(RuntimeItem {
            id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            icon: manifest.icon,
            website: manifest.website,
            description: manifest.description,
            features: manifest.features,
            is_installed,
            installed_path,
            active_acceleration: active_accel.clone(),
            installed_acceleration: installed_accel,
            available_accelerations: avail_options,
            install_progress: progress,
        });
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    Json(items)
}

/// Trigger background installation/download for a runtime
#[utoipa::path(
    post,
    path = "/api/runtimes/{id}/install",
    tag = "Runtimes",
    params(
        ("id" = String, Path, description = "Runtime ID (e.g. llama-cpp)")
    ),
    request_body(content = Option<InstallRuntimeRequest>, description = "Optional hardware override"),
    responses(
        (status = 200, description = "Installation started in background", body = OperationResponse),
        (status = 400, description = "Runtimes directory not configured", body = String)
    )
)]
pub async fn install_runtime_handler(
    State(state): State<AppState>,
    Path(runtime_id): Path<String>,
    payload: Option<Json<InstallRuntimeRequest>>,
) -> Result<Json<OperationResponse>, (StatusCode, String)> {
    let (runtimes_dir, configured_hardware) = {
        let config = state.config.lock().unwrap();
        let dir = config.runtimes.as_deref().unwrap_or("").to_string();
        if dir.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Runtimes directory is not configured in config.yaml".to_string(),
            ));
        }
        let hw = config.hardware.clone();
        (crate::config::expand_tilde(dir), hw)
    };

    let hardware_override = payload
        .and_then(|p| p.hardware.clone())
        .or(configured_hardware);

    let installer_mgr = state.installer_manager.clone();
    let rid_clone = runtime_id.clone();

    // Spawn download and extraction in background task
    tokio::spawn(async move {
        if let Err(err) = install_runtime(
            installer_mgr,
            runtimes_dir,
            rid_clone.clone(),
            hardware_override,
        )
        .await
        {
            tracing::error!("Failed to install runtime '{}': {}", rid_clone, err);
        }
    });

    Ok(Json(OperationResponse {
        success: true,
        message: format!("Installation initiated for runtime '{}'", runtime_id),
    }))
}

/// Real-time SSE stream of download/extraction progress for a runtime
#[utoipa::path(
    get,
    path = "/api/runtimes/{id}/install/stream",
    tag = "Runtimes",
    params(
        ("id" = String, Path, description = "Runtime ID (e.g. llama-cpp)")
    ),
    responses(
        (status = 200, description = "SSE progress events stream", content_type = "text/event-stream")
    )
)]
pub async fn install_stream_handler(
    State(state): State<AppState>,
    Path(runtime_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let installer_mgr = state.installer_manager.clone();
    let mut receiver = installer_mgr.subscribe();
    let target_id = runtime_id.clone();

    let stream = stream! {
        // Yield initial progress state if exists
        if let Some(initial) = installer_mgr.get_progress(&target_id) {
            if let Ok(json_str) = serde_json::to_string(&initial) {
                yield Ok(Event::default().data(json_str));
            }
        }

        while let Ok(progress) = receiver.recv().await {
            if progress.runtime_id == target_id {
                let status = progress.status.clone();
                if let Ok(json_str) = serde_json::to_string(&progress) {
                    yield Ok(Event::default().data(json_str));
                }
                if status == "completed" || status == "error" {
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/// Uninstall an installed runtime binary
#[utoipa::path(
    delete,
    path = "/api/runtimes/{id}",
    tag = "Runtimes",
    params(
        ("id" = String, Path, description = "Runtime ID (e.g. llama-cpp)")
    ),
    responses(
        (status = 200, description = "Runtime uninstalled successfully", body = OperationResponse),
        (status = 400, description = "Runtimes directory not configured", body = String),
        (status = 500, description = "Failed to uninstall runtime", body = String)
    )
)]
pub async fn uninstall_runtime_handler(
    State(state): State<AppState>,
    Path(runtime_id): Path<String>,
) -> Result<Json<OperationResponse>, (StatusCode, String)> {
    let runtimes_dir = {
        let config = state.config.lock().unwrap();
        let dir = config.runtimes.as_deref().unwrap_or("").to_string();
        if dir.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Runtimes directory is not configured in config.yaml".to_string(),
            ));
        }
        crate::config::expand_tilde(dir)
    };

    uninstall_runtime(&runtimes_dir, &runtime_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Reset progress in installer manager
    state.installer_manager.update_progress(InstallProgress {
        runtime_id: runtime_id.clone(),
        status: "idle".to_string(),
        percent: 0.0,
        speed_mbps: 0.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        error_message: None,
        message: None,
    });

    Ok(Json(OperationResponse {
        success: true,
        message: format!("Runtime '{}' uninstalled successfully", runtime_id),
    }))
}
