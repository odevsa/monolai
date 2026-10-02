use crate::db::models::{
    delete_model, get_all_models, get_model_by_id, insert_model, update_model, CreateModelPayload,
    ModelRecord,
};
use crate::runtimes::model_scanner::{scan_models, ModelItem};
use crate::runtimes::process_manager::{
    get_all_running_status, load_model_process, swap_model_process, unload_all_model_processes,
    unload_model_process, RunningModelStatus,
};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::path::PathBuf;

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
    let status = state.config_status.lock().unwrap();
    let models_dir = if let Some(ref dir_str) = status.models_dir {
        PathBuf::from(dir_str)
    } else {
        crate::config::get_default_models_dir()
    };

    let items = scan_models(models_dir);
    Json(items)
}

/// List all registered models from database
#[utoipa::path(
    get,
    path = "/api/models",
    tag = "Models",
    responses(
        (status = 200, description = "List of registered models", body = Vec<ModelRecord>),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_models_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<ModelRecord>>, (StatusCode, String)> {
    get_all_models(&state.db)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
    Json(mut payload): Json<CreateModelPayload>,
) -> Result<Json<ModelRecord>, (StatusCode, String)> {
    let id = payload.id.trim().to_string();

    if id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Model ID cannot be empty.".into()));
    }

    if id.contains(' ') || id.contains('\t') || id.contains('\n') {
        return Err((
            StatusCode::BAD_REQUEST,
            "Model ID cannot contain spaces.".into(),
        ));
    }

    if let Ok(Some(_)) = get_model_by_id(&state.db, &id).await {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("A model with ID '{}' already exists.", id),
        ));
    }

    payload.id = id;

    insert_model(&state.db, &payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
) -> Result<Json<ModelRecord>, (StatusCode, String)> {
    if get_model_by_id(&state.db, &id).await.ok().flatten().is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Model with ID '{}' not found.", id),
        ));
    }

    update_model(&state.db, &id, &payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
) -> Result<StatusCode, (StatusCode, String)> {
    delete_model(&state.db, &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
) -> Result<Json<RunningModelStatus>, (StatusCode, String)> {
    let config = state.config.lock().unwrap().clone();
    load_model_process(&state.process_manager, &state.db, &config, &id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
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
) -> Result<Json<RunningModelStatus>, (StatusCode, String)> {
    let config = state.config.lock().unwrap().clone();
    swap_model_process(&state.process_manager, &state.db, &config, &id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
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
) -> Result<StatusCode, (StatusCode, String)> {
    unload_model_process(&state.process_manager, &state.db, &id)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
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
pub async fn unload_all_models_handler(
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    unload_all_model_processes(&state.process_manager, &state.db)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
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
    let list = get_all_running_status(&state.process_manager).await;
    Json(list)
}


