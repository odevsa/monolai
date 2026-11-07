use crate::core::error::AppResult;
use crate::domain::{AudioGenerationRecord, CreateAudioGenerationPayload};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AudioListQuery {
    #[serde(rename = "type")]
    pub audio_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List audio generations history
#[utoipa::path(
    get,
    path = "/api/audio",
    tag = "Audio",
    params(
        ("type" = Option<String>, Query, description = "Filter by audio type ('tts' or 'asr')"),
        ("limit" = Option<i64>, Query, description = "Maximum number of items to return"),
        ("offset" = Option<i64>, Query, description = "Number of items to skip")
    ),
    responses(
        (status = 200, description = "List of audio generations", body = Vec<AudioGenerationRecord>),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_audio_handler(
    State(state): State<AppState>,
    Query(query): Query<AudioListQuery>,
) -> AppResult<Json<Vec<AudioGenerationRecord>>> {
    let items = state
        .audio_service
        .list_audio(query.audio_type.as_deref(), query.limit, query.offset)
        .await?;
    Ok(Json(items))
}

/// Get an audio generation record by ID
#[utoipa::path(
    get,
    path = "/api/audio/{id}",
    tag = "Audio",
    params(
        ("id" = String, Path, description = "Audio record ID")
    ),
    responses(
        (status = 200, description = "Audio generation record", body = AudioGenerationRecord),
        (status = 404, description = "Record not found"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_audio_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Result<Json<AudioGenerationRecord>, StatusCode>> {
    let item = state.audio_service.get_audio_by_id(&id).await?;
    match item {
        Some(record) => Ok(Ok(Json(record))),
        None => Ok(Err(StatusCode::NOT_FOUND)),
    }
}

/// Create a new audio generation history record
#[utoipa::path(
    post,
    path = "/api/audio",
    tag = "Audio",
    request_body = CreateAudioGenerationPayload,
    responses(
        (status = 201, description = "Audio generation record saved", body = AudioGenerationRecord),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn create_audio_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateAudioGenerationPayload>,
) -> AppResult<(StatusCode, Json<AudioGenerationRecord>)> {
    let record = state.audio_service.create_audio(payload).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// Delete an audio generation record by ID
#[utoipa::path(
    delete,
    path = "/api/audio/{id}",
    tag = "Audio",
    params(
        ("id" = String, Path, description = "Audio record ID")
    ),
    responses(
        (status = 204, description = "Audio record deleted successfully"),
        (status = 404, description = "Record not found"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn delete_audio_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let deleted = state.audio_service.delete_audio(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

/// Clear all audio generations history
#[utoipa::path(
    delete,
    path = "/api/audio",
    tag = "Audio",
    responses(
        (status = 204, description = "All audio records deleted successfully"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn clear_audio_handler(State(state): State<AppState>) -> AppResult<StatusCode> {
    state.audio_service.clear_audio().await?;
    Ok(StatusCode::NO_CONTENT)
}

