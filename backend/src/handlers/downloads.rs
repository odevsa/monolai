use crate::core::error::AppResult;
use crate::domain::{DownloadRecord, EnqueueDownloadPayload};
use crate::state::AppState;
use async_stream::stream;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        Json,
    },
};
use futures_util::stream::Stream;
use std::{convert::Infallible, time::Duration};

#[utoipa::path(
    get,
    path = "/api/downloads",
    tag = "Downloads",
    responses(
        (status = 200, description = "List of all downloads", body = Vec<DownloadRecord>)
    )
)]
pub async fn get_downloads_handler(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<DownloadRecord>>> {
    let list = state.download_service.get_all().await?;
    Ok(Json(list))
}

#[utoipa::path(
    post,
    path = "/api/downloads",
    tag = "Downloads",
    request_body = EnqueueDownloadPayload,
    responses(
        (status = 201, description = "Download enqueued successfully", body = DownloadRecord)
    )
)]
pub async fn enqueue_download_handler(
    State(state): State<AppState>,
    Json(payload): Json<EnqueueDownloadPayload>,
) -> AppResult<(StatusCode, Json<DownloadRecord>)> {
    let record = state.download_service.enqueue_download(payload).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

#[utoipa::path(
    post,
    path = "/api/downloads/{id}/pause",
    tag = "Downloads",
    params(
        ("id" = String, Path, description = "Download Task ID")
    ),
    responses(
        (status = 200, description = "Download paused successfully")
    )
)]
pub async fn pause_download_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.download_service.pause_download(&id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/downloads/{id}/resume",
    tag = "Downloads",
    params(
        ("id" = String, Path, description = "Download Task ID")
    ),
    responses(
        (status = 200, description = "Download resumed successfully")
    )
)]
pub async fn resume_download_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.download_service.resume_download(&id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/downloads/{id}/cancel",
    tag = "Downloads",
    params(
        ("id" = String, Path, description = "Download Task ID")
    ),
    responses(
        (status = 200, description = "Download cancelled successfully")
    )
)]
pub async fn cancel_download_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.download_service.cancel_download(&id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/api/downloads/{id}",
    tag = "Downloads",
    params(
        ("id" = String, Path, description = "Download Task ID")
    ),
    responses(
        (status = 204, description = "Download deleted successfully")
    )
)]
pub async fn delete_download_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.download_service.delete_download(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/downloads/stream",
    tag = "Downloads",
    responses(
        (status = 200, description = "SSE stream of download tasks updates", content_type = "text/event-stream")
    )
)]
pub async fn downloads_stream_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let download_service = state.download_service.clone();
    let mut receiver = download_service.subscribe();
    let initial_records = download_service.get_all().await.unwrap_or_default();

    let stream = stream! {
        if let Ok(json_str) = serde_json::to_string(&initial_records) {
            yield Ok(Event::default().data(json_str));
        }

        loop {
            match receiver.recv().await {
                Ok(records) => {
                    if let Ok(json_str) = serde_json::to_string(&records) {
                        yield Ok(Event::default().data(json_str));
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    let current = download_service.get_all().await.unwrap_or_default();
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
