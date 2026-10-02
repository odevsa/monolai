use crate::db::settings::{delete_setting, get_all_settings, get_setting, set_setting};
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
) -> Result<Json<HashMap<String, String>>, (StatusCode, String)> {
    get_all_settings(&state.db)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
) -> Result<Json<Option<String>>, (StatusCode, String)> {
    get_setting(&state.db, &key)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
) -> Result<Json<HashMap<String, String>>, (StatusCode, String)> {
    for (key, value) in &payload {
        set_setting(&state.db, key, value)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
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
) -> Result<StatusCode, (StatusCode, String)> {
    delete_setting(&state.db, &key)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
