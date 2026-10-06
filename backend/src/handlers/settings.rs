use crate::core::error::AppResult;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::collections::HashMap;

/// Get all key-value application settings
#[utoipa::path(
    get,
    path = "/api/settings",
    tag = "Settings",
    responses(
        (status = 200, description = "Map of all settings", body = HashMap<String, String>),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_all_settings_handler(
    State(state): State<AppState>,
) -> AppResult<Json<HashMap<String, String>>> {
    let settings = state.setting_repo.get_all().await?;
    Ok(Json(settings))
}

/// Get a specific setting by key
#[utoipa::path(
    get,
    path = "/api/settings/{key}",
    tag = "Settings",
    params(
        ("key" = String, Path, description = "Setting key name")
    ),
    responses(
        (status = 200, description = "Setting value if present", body = Option<String>),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_setting_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> AppResult<Json<Option<String>>> {
    let setting = state.setting_repo.get(&key).await?;
    Ok(Json(setting))
}

/// Upsert multiple key-value application settings
#[utoipa::path(
    post,
    path = "/api/settings",
    tag = "Settings",
    request_body(content = HashMap<String, String>, description = "Key-value pairs to set"),
    responses(
        (status = 200, description = "Settings saved successfully", body = HashMap<String, String>),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn update_settings_handler(
    State(state): State<AppState>,
    Json(payload): Json<HashMap<String, String>>,
) -> AppResult<Json<HashMap<String, String>>> {
    for (key, value) in &payload {
        state.setting_repo.set(key, value).await?;
    }
    Ok(Json(payload))
}

/// Delete a specific setting by key
#[utoipa::path(
    delete,
    path = "/api/settings/{key}",
    tag = "Settings",
    params(
        ("key" = String, Path, description = "Setting key name")
    ),
    responses(
        (status = 204, description = "Setting deleted successfully"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn delete_setting_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> AppResult<StatusCode> {
    state.setting_repo.delete(&key).await?;
    Ok(StatusCode::NO_CONTENT)
}
