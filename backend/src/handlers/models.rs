use crate::core::error::AppResult;
use crate::domain::{CreateModelPayload, ModelItem, ModelRecord, RunningModelStatus};
use crate::state::AppState;
use async_stream::stream;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        Json,
    },
};
use futures_util::stream::Stream;
use serde::Deserialize;
use std::{convert::Infallible, time::Duration};

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct GetModelsQuery {
    pub all: Option<bool>,
    pub include_missing: Option<bool>,
}

/// List scanned model files (GGUF, Safetensors) in models directory
#[utoipa::path(
    get,
    path = "/api/models/available",
    tag = "Models",
    responses(
        (status = 200, description = "List of scanned model files", body = Vec<ModelItem>)
    )
)]
pub async fn available_models_handler(State(state): State<AppState>) -> Json<Vec<ModelItem>> {
    let items = state.model_service.get_available_models().await;
    Json(items)
}

/// List registered models from database
#[utoipa::path(
    get,
    path = "/api/models",
    tag = "Models",
    params(
        ("all" = Option<bool>, Query, description = "Include models whose files are missing on disk (default: false)")
    ),
    responses(
        (status = 200, description = "List of registered models", body = Vec<ModelRecord>),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_models_handler(
    State(state): State<AppState>,
    Query(query): Query<GetModelsQuery>,
) -> AppResult<Json<Vec<ModelRecord>>> {
    let include_all = query.all.unwrap_or(false) || query.include_missing.unwrap_or(false);
    let models = state.model_service.get_models(include_all).await?;
    Ok(Json(models))
}

/// Register a new model configuration
#[utoipa::path(
    post,
    path = "/api/models",
    tag = "Models",
    request_body = CreateModelPayload,
    responses(
        (status = 200, description = "Model registered successfully", body = ModelRecord),
        (status = 400, description = "Validation error", body = String),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn create_model_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateModelPayload>,
) -> AppResult<Json<ModelRecord>> {
    let model = state.model_service.create_model(payload).await?;
    Ok(Json(model))
}

/// Update an existing registered model
#[utoipa::path(
    put,
    path = "/api/models/{id}",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    request_body = CreateModelPayload,
    responses(
        (status = 200, description = "Model updated successfully", body = ModelRecord),
        (status = 404, description = "Model not found", body = String),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn update_model_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateModelPayload>,
) -> AppResult<Json<ModelRecord>> {
    let model = state.model_service.update_model(&id, payload).await?;
    Ok(Json(model))
}

/// Delete a registered model
#[utoipa::path(
    delete,
    path = "/api/models/{id}",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    responses(
        (status = 204, description = "Model deleted successfully"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn delete_model_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.model_service.delete_model(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Load a model process by ID
#[utoipa::path(
    post,
    path = "/api/models/{id}/load",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    responses(
        (status = 200, description = "Model process status", body = RunningModelStatus),
        (status = 500, description = "Failed to load model process", body = String)
    )
)]
pub async fn load_model_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<RunningModelStatus>> {
    let status = state.model_service.load_model(&id).await?;
    Ok(Json(status))
}

/// Swap to a specific model (unloads others, then loads target model)
#[utoipa::path(
    post,
    path = "/api/models/{id}/swap",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    responses(
        (status = 200, description = "Model swapped successfully", body = RunningModelStatus),
        (status = 500, description = "Failed to swap model process", body = String)
    )
)]
pub async fn swap_model_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<RunningModelStatus>> {
    let status = state.model_service.swap_model(&id).await?;
    Ok(Json(status))
}

/// Unload a model process by ID
#[utoipa::path(
    post,
    path = "/api/models/{id}/unload",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    responses(
        (status = 200, description = "Model unloaded successfully"),
        (status = 500, description = "Failed to unload model process", body = String)
    )
)]
pub async fn unload_model_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.model_service.unload_model(&id).await?;
    Ok(StatusCode::OK)
}

/// Unload all running model processes
#[utoipa::path(
    post,
    path = "/api/models/unload-all",
    tag = "Models",
    responses(
        (status = 200, description = "All model processes unloaded successfully"),
        (status = 500, description = "Failed to unload model processes", body = String)
    )
)]
pub async fn unload_all_models_handler(State(state): State<AppState>) -> AppResult<StatusCode> {
    state.model_service.unload_all_models().await?;
    Ok(StatusCode::OK)
}

/// Get running state of all active model processes
#[utoipa::path(
    get,
    path = "/api/state",
    tag = "Models",
    responses(
        (status = 200, description = "List of currently running model processes", body = Vec<RunningModelStatus>)
    )
)]
pub async fn get_running_models_handler(
    State(state): State<AppState>,
) -> Json<Vec<RunningModelStatus>> {
    let list = state.model_service.get_running_models().await;
    Json(list)
}

/// Real-time SSE stream of running model states
#[utoipa::path(
    get,
    path = "/api/state/stream",
    tag = "Models",
    responses(
        (status = 200, description = "SSE stream of running model states", content_type = "text/event-stream")
    )
)]
pub async fn running_models_stream_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let model_service = state.model_service.clone();
    let mut receiver = model_service.subscribe_state();
    let initial_status = model_service.get_running_models().await;

    let stream = stream! {
        if let Ok(json_str) = serde_json::to_string(&initial_status) {
            yield Ok(Event::default().data(json_str));
        }

        loop {
            match receiver.recv().await {
                Ok(status) => {
                    if let Ok(json_str) = serde_json::to_string(&status) {
                        yield Ok(Event::default().data(json_str));
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    let current = model_service.get_running_models().await;
                    if let Ok(json_str) = serde_json::to_string(&current) {
                        yield Ok(Event::default().data(json_str));
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
