use crate::core::error::AppResult;
use crate::domain::RuntimeItem;
use crate::state::AppState;
use async_stream::stream;
use axum::{
    extract::{Path, State},
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
    let items = state.runtime_service.get_runtimes().await;
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
) -> AppResult<Json<OperationResponse>> {
    let hardware_override = payload.and_then(|p| p.hardware.clone());
    state.runtime_service.install(&runtime_id, hardware_override).await?;

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
    let mut receiver = state.runtime_service.subscribe();
    let initial_opt = state.runtime_service.get_progress(&runtime_id);
    let target_id = runtime_id.clone();

    let stream = stream! {
        if let Some(initial) = initial_opt {
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
) -> AppResult<Json<OperationResponse>> {
    state.runtime_service.uninstall(&runtime_id).await?;

    Ok(Json(OperationResponse {
        success: true,
        message: format!("Runtime '{}' uninstalled successfully", runtime_id),
    }))
}
