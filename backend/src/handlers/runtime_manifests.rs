use crate::runtimes::manifest_loader::{
    get_manifest_for_runtime, get_runtime_manifests, RuntimeManifest,
};
use axum::{
    extract::Path,
    http::StatusCode,
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
pub async fn get_runtime_manifests_handler() -> Json<Vec<RuntimeManifest>> {
    Json(get_runtime_manifests())
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
    Path(id): Path<String>,
) -> Result<Json<RuntimeManifest>, (StatusCode, String)> {
    if let Some(manifest) = get_manifest_for_runtime(&id) {
        Ok(Json(manifest))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!("Runtime manifest for '{}' not found", id),
        ))
    }
}
