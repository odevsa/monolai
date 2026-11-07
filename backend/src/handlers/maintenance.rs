use crate::core::error::AppError;
use crate::domain::MaintenanceResultDto;
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

#[utoipa::path(
    post,
    path = "/api/maintenance/clean-database",
    responses(
        (status = 200, description = "Wipe database and rebuild migrations", body = MaintenanceResultDto),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "Maintenance"
)]
pub async fn clean_database_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.maintenance_service.clean_database().await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/maintenance/clean-storage",
    responses(
        (status = 200, description = "Purge models and runtimes storage folders", body = MaintenanceResultDto),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "Maintenance"
)]
pub async fn clean_storage_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.maintenance_service.clean_storage().await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/maintenance/factory-reset",
    responses(
        (status = 200, description = "Factory reset database, storage, and configurations", body = MaintenanceResultDto),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "Maintenance"
)]
pub async fn factory_reset_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.maintenance_service.factory_reset().await?;
    Ok(Json(result))
}
