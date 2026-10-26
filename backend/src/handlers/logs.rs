use async_stream::stream;
use axum::{
    extract::{Query, State},
    http::header,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;

use crate::domain::{LogEntry, LogFilterQuery};
use crate::state::AppState;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LogOperationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ClearLogsQuery {
    pub service: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct DownloadLogsQuery {
    pub service: Option<String>,
}

/// Query in-memory log entries with filtering and limit
#[utoipa::path(
    get,
    path = "/api/logs",
    params(LogFilterQuery),
    responses(
        (status = 200, description = "Filtered log entries", body = Vec<LogEntry>)
    )
)]
pub async fn get_logs_handler(
    State(state): State<AppState>,
    Query(filter): Query<LogFilterQuery>,
) -> Json<Vec<LogEntry>> {
    let entries = state.log_service.get_entries(&filter).await;
    Json(entries)
}

/// List all available log services (e.g. "all", "monolai", "llama.cpp 1", etc.)
#[utoipa::path(
    get,
    path = "/api/logs/services",
    responses(
        (status = 200, description = "List of services with available logs", body = Vec<String>)
    )
)]
pub async fn get_log_services_handler(State(state): State<AppState>) -> Json<Vec<String>> {
    let services = state.log_service.get_services().await;
    Json(services)
}

/// Real-time Server-Sent Events (SSE) stream of log entries
#[utoipa::path(
    get,
    path = "/api/logs/stream",
    responses(
        (status = 200, description = "Real-time log events stream", content_type = "text/event-stream")
    )
)]
pub async fn logs_stream_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut receiver = state.log_service.subscribe();

    let stream = stream! {
        while let Ok(entry) = receiver.recv().await {
            if let Ok(json_str) = serde_json::to_string(&entry) {
                yield Ok(Event::default().data(json_str));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/// Clears logs from in-memory buffer and truncates disk log files
#[utoipa::path(
    delete,
    path = "/api/logs",
    params(ClearLogsQuery),
    responses(
        (status = 200, description = "Logs cleared successfully", body = LogOperationResponse)
    )
)]
pub async fn clear_logs_handler(
    State(state): State<AppState>,
    Query(query): Query<ClearLogsQuery>,
) -> Json<LogOperationResponse> {
    state.log_service.clear_logs(query.service.as_deref()).await;
    Json(LogOperationResponse {
        success: true,
        message: "Logs cleared successfully".to_string(),
    })
}

/// Download log file as a text attachment
#[utoipa::path(
    get,
    path = "/api/logs/download",
    params(DownloadLogsQuery),
    responses(
        (status = 200, description = "Log file text attachment", content_type = "text/plain")
    )
)]
pub async fn download_logs_handler(
    State(state): State<AppState>,
    Query(query): Query<DownloadLogsQuery>,
) -> Response {
    let (filename, content) = state
        .log_service
        .get_download_content(query.service.as_deref())
        .await;

    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        content,
    )
        .into_response()
}
