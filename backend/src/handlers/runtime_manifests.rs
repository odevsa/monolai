use crate::core::error::{AppError, AppResult};
use crate::domain::RuntimeManifest;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    response::Json,
};

/// Get all available runtime manifests
#[utoipa::path(
    get,
    path = "/api/runtime-manifests",
    tag = "Runtimes",
    responses(
        (status = 200, description = "List of all supported runtime manifests", body = Vec<RuntimeManifest>)
    )
)]
pub async fn get_runtime_manifests_handler(
    State(state): State<AppState>,
) -> Json<Vec<RuntimeManifest>> {
    Json(state.runtime_service.manifests())
}

/// Get manifest for a specific runtime ID
#[utoipa::path(
    get,
    path = "/api/runtime-manifests/{id}",
    tag = "Runtimes",
    params(
        ("id" = String, Path, description = "Runtime ID (e.g. llama-cpp, vllm, ollama)")
    ),
    responses(
        (status = 200, description = "Runtime manifest details", body = RuntimeManifest),
        (status = 404, description = "Runtime manifest not found", body = String)
    )
)]
pub async fn get_runtime_manifest_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<RuntimeManifest>> {
    if let Some(manifest) = state.runtime_service.manifest_by_id(&id) {
        Ok(Json(manifest))
    } else {
        Err(AppError::not_found(format!("Runtime manifest for '{}' not found", id)))
    }
}
