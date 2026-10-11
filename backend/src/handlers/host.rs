use crate::domain::SysInfoResponse;
use crate::state::AppState;
use async_stream::stream;
use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        Json,
    },
};
use futures_util::stream::Stream;
use std::{convert::Infallible, time::Duration};

/// Get snapshot of system host information (CPU, RAM, OS, GPU)
#[utoipa::path(
    get,
    path = "/api/host",
    tag = "Host",
    responses(
        (status = 200, description = "Host system metrics snapshot", body = SysInfoResponse)
    )
)]
pub async fn sysinfo_handler(State(state): State<AppState>) -> Json<SysInfoResponse> {
    Json(state.host_service.get_sysinfo())
}

/// Real-time SSE stream of system host performance metrics
#[utoipa::path(
    get,
    path = "/api/host/usage",
    tag = "Host",
    responses(
        (status = 200, description = "SSE metrics stream tick every 2 seconds", content_type = "text/event-stream")
    )
)]
pub async fn sysinfo_stream_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let host_service = state.host_service.clone();

    let stream = stream! {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let tick = host_service.get_metrics_tick();
            if let Ok(json_str) = serde_json::to_string(&tick) {
                yield Ok(Event::default().data(json_str));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
