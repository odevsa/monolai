use crate::core::error::AppResult;
use crate::domain::{CreateGeneratedImagePayload, GeneratedImageRecord};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ImageListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List generated images history
#[utoipa::path(
    get,
    path = "/api/images",
    tag = "Images",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of images to return"),
        ("offset" = Option<i64>, Query, description = "Number of items to skip")
    ),
    responses(
        (status = 200, description = "List of generated images", body = Vec<GeneratedImageRecord>),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_images_handler(
    State(state): State<AppState>,
    Query(query): Query<ImageListQuery>,
) -> AppResult<Json<Vec<GeneratedImageRecord>>> {
    let images = state.image_service.list_images(query.limit, query.offset).await?;
    Ok(Json(images))
}

/// Get a generated image by ID
#[utoipa::path(
    get,
    path = "/api/images/{id}",
    tag = "Images",
    params(
        ("id" = String, Path, description = "Image ID")
    ),
    responses(
        (status = 200, description = "Generated image record", body = GeneratedImageRecord),
        (status = 404, description = "Image not found"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_image_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Result<Json<GeneratedImageRecord>, StatusCode>> {
    let image = state.image_service.get_image_by_id(&id).await?;
    match image {
        Some(img) => Ok(Ok(Json(img))),
        None => Ok(Err(StatusCode::NOT_FOUND)),
    }
}

/// Create a new generated image history record
#[utoipa::path(
    post,
    path = "/api/images",
    tag = "Images",
    request_body = CreateGeneratedImagePayload,
    responses(
        (status = 201, description = "Image saved successfully", body = GeneratedImageRecord),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn create_image_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateGeneratedImagePayload>,
) -> AppResult<(StatusCode, Json<GeneratedImageRecord>)> {
    let record = state.image_service.create_image(payload).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// Delete a generated image record by ID
#[utoipa::path(
    delete,
    path = "/api/images/{id}",
    tag = "Images",
    params(
        ("id" = String, Path, description = "Image ID")
    ),
    responses(
        (status = 204, description = "Image deleted successfully"),
        (status = 404, description = "Image not found"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn delete_image_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let deleted = state.image_service.delete_image(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

/// Clear all generated images history
#[utoipa::path(
    delete,
    path = "/api/images",
    tag = "Images",
    responses(
        (status = 204, description = "All images deleted successfully"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn clear_images_handler(State(state): State<AppState>) -> AppResult<StatusCode> {
    state.image_service.clear_images().await?;
    Ok(StatusCode::NO_CONTENT)
}

