use crate::core::error::AppResult;
use crate::domain::{
    CreatePresetPayload, ModelItem, ModelPreset, ModelRecord, RunningModelStatus,
    RuntimeManifest, UpdatePresetPayload,
};
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

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct DeleteModelQuery {
    pub delete_file: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct CompatibleRuntimesQuery {
    pub modality: Option<String>,
    pub format: Option<String>,
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

/// List registered models with their presets
#[utoipa::path(
    get,
    path = "/api/models",
    tag = "Models",
    params(
        ("all" = Option<bool>, Query, description = "Include models whose files are missing on disk (default: false)")
    ),
    responses(
        (status = 200, description = "List of registered models", body = Vec<ModelRecord>)
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

/// Get a specific registered model by ID
#[utoipa::path(
    get,
    path = "/api/models/{id}",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    responses(
        (status = 200, description = "Registered model details", body = ModelRecord)
    )
)]
pub async fn get_model_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ModelRecord>> {
    let model = state.model_service.get_model_by_id(&id).await?;
    Ok(Json(model))
}

/// Delete a registered model (optionally deleting physical file)
#[utoipa::path(
    delete,
    path = "/api/models/{id}",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID"),
        ("delete_file" = Option<bool>, Query, description = "Delete file from disk as well")
    ),
    responses(
        (status = 204, description = "Model deleted successfully")
    )
)]
pub async fn delete_model_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<DeleteModelQuery>,
) -> AppResult<StatusCode> {
    let delete_file = query.delete_file.unwrap_or(true);
    state
        .model_service
        .delete_model(&id, delete_file)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get presets configured for a specific model
#[utoipa::path(
    get,
    path = "/api/models/{id}/presets",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    responses(
        (status = 200, description = "List of model presets", body = Vec<ModelPreset>)
    )
)]
pub async fn get_model_presets_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<ModelPreset>>> {
    let presets = state.model_service.get_presets(&id).await?;
    Ok(Json(presets))
}

/// Create a new preset for a model
#[utoipa::path(
    post,
    path = "/api/models/{id}/presets",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    request_body = CreatePresetPayload,
    responses(
        (status = 201, description = "Preset created successfully", body = ModelPreset)
    )
)]
pub async fn create_model_preset_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreatePresetPayload>,
) -> AppResult<(StatusCode, Json<ModelPreset>)> {
    let preset = state.model_service.create_preset(&id, payload).await?;
    Ok((StatusCode::CREATED, Json(preset)))
}

/// Update an existing model preset
#[utoipa::path(
    put,
    path = "/api/models/presets/{preset_id}",
    tag = "Models",
    params(
        ("preset_id" = String, Path, description = "Preset ID")
    ),
    request_body = UpdatePresetPayload,
    responses(
        (status = 200, description = "Preset updated successfully", body = ModelPreset)
    )
)]
pub async fn update_model_preset_handler(
    State(state): State<AppState>,
    Path(preset_id): Path<String>,
    Json(payload): Json<UpdatePresetPayload>,
) -> AppResult<Json<ModelPreset>> {
    let preset = state.model_service.update_preset(&preset_id, payload).await?;
    Ok(Json(preset))
}

/// Delete a model preset
#[utoipa::path(
    delete,
    path = "/api/models/presets/{preset_id}",
    tag = "Models",
    params(
        ("preset_id" = String, Path, description = "Preset ID")
    ),
    responses(
        (status = 204, description = "Preset deleted successfully")
    )
)]
pub async fn delete_model_preset_handler(
    State(state): State<AppState>,
    Path(preset_id): Path<String>,
) -> AppResult<StatusCode> {
    state.model_service.delete_preset(&preset_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Set a preset as default for its parent model
#[utoipa::path(
    post,
    path = "/api/models/{id}/presets/{preset_id}/set-default",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID"),
        ("preset_id" = String, Path, description = "Preset ID")
    ),
    responses(
        (status = 200, description = "Default preset set successfully")
    )
)]
pub async fn set_default_model_preset_handler(
    State(state): State<AppState>,
    Path((id, preset_id)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    state.model_service.set_default_preset(&id, &preset_id).await?;
    Ok(StatusCode::OK)
}

/// Query compatible runtimes for a given modality and format
#[utoipa::path(
    get,
    path = "/api/runtimes/compatible",
    tag = "Runtimes",
    params(
        ("modality" = Option<String>, Query, description = "Model modality"),
        ("format" = Option<String>, Query, description = "Model format")
    ),
    responses(
        (status = 200, description = "List of compatible runtimes", body = Vec<RuntimeManifest>)
    )
)]
pub async fn get_compatible_runtimes_handler(
    State(state): State<AppState>,
    Query(query): Query<CompatibleRuntimesQuery>,
) -> Json<Vec<RuntimeManifest>> {
    let runtimes = state
        .model_service
        .get_compatible_runtimes(query.modality.as_deref(), query.format.as_deref())
        .await;
    Json(runtimes)
}

/// Load a model process by ID
#[utoipa::path(
    post,
    path = "/api/models/{id}/load",
    tag = "Models",
    params(
        ("id" = String, Path, description = "Model ID or preset ID")
    ),
    responses(
        (status = 200, description = "Model process status", body = RunningModelStatus)
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
        ("id" = String, Path, description = "Model ID or preset ID")
    ),
    responses(
        (status = 200, description = "Model swapped successfully", body = RunningModelStatus)
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
        (status = 200, description = "Model unloaded successfully")
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
        (status = 200, description = "All model processes unloaded successfully")
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
