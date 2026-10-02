use crate::state::AppState;
use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        Json,
    },
};
use futures_util::stream::Stream;
use serde::Serialize;
use std::{
    convert::Infallible,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use sysinfo::System;

#[derive(Serialize, utoipa::ToSchema)]
pub struct CpuInfo {
    pub usage: f32,
    pub cores: usize,
    pub brand: String,
    pub frequency_mhz: u64,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RamInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub percentage: f32,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct OsInfo {
    pub name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub hostname: String,
    pub uptime_seconds: u64,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SysInfoResponse {
    pub cpu: CpuInfo,
    pub ram: RamInfo,
    pub os: OsInfo,
    #[schema(value_type = Option<Object>)]
    pub gpu: Option<serde_json::Value>,
    pub timestamp: u64,
}

#[derive(Serialize, Clone, utoipa::ToSchema)]
pub struct HostMetricsTick {
    pub cpu_usage: f32,
    pub ram_used_bytes: u64,
    pub ram_total_bytes: u64,
    pub ram_free_bytes: u64,
    pub ram_percentage: f32,
    pub gpu_usage: Option<f32>,
    pub timestamp: u64,
}

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
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_cpu();
    sys.refresh_memory();

    let cpus = sys.cpus();
    let global_cpu_usage = sys.global_cpu_info().cpu_usage();
    let cpu_brand = cpus
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_freq = cpus.first().map(|c| c.frequency()).unwrap_or(0);

    let total_ram = sys.total_memory();
    let used_ram = sys.used_memory();
    let free_ram = sys.free_memory();
    let ram_pct = if total_ram > 0 {
        (used_ram as f32 / total_ram as f32) * 100.0
    } else {
        0.0
    };

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let sys_name = System::name().unwrap_or_else(|| "Linux".to_string());
    let kernel_ver = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let os_ver = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let host_name = System::host_name().unwrap_or_else(|| "localhost".to_string());
    let uptime = System::uptime();

    Json(SysInfoResponse {
        cpu: CpuInfo {
            usage: global_cpu_usage,
            cores: cpus.len(),
            brand: cpu_brand,
            frequency_mhz: cpu_freq,
        },
        ram: RamInfo {
            total_bytes: total_ram,
            used_bytes: used_ram,
            free_bytes: free_ram,
            percentage: ram_pct,
        },
        os: OsInfo {
            name: sys_name,
            kernel_version: kernel_ver,
            os_version: os_ver,
            hostname: host_name,
            uptime_seconds: uptime,
        },
        gpu: None,
        timestamp: ts,
    })
}

/// Real-time Server-Sent Events (SSE) stream of system resource usage
#[utoipa::path(
    get,
    path = "/api/host/usage",
    tag = "Host",
    responses(
        (status = 200, description = "SSE metrics stream (CPU, RAM, GPU) emitted every 500ms", content_type = "text/event-stream")
    )
)]
pub async fn sysinfo_stream_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        // TEST DISCONNECT: Uncomment line below to track ticks
        // let mut tick_count = 0;

        loop {
            interval.tick().await;

            // TEST DISCONNECT: Uncomment block below to simulate SSE drop after 5 seconds (10 ticks at 500ms)
            /*
            tick_count += 1;
            if tick_count >= 10 {
                tracing::info!("Simulating SSE disconnect after 5s");
                break;
            }
            */

            let tick = {
                let mut sys = state.sys.lock().unwrap();
                sys.refresh_cpu();
                sys.refresh_memory();

                let global_cpu_usage = sys.global_cpu_info().cpu_usage();
                let total_ram = sys.total_memory();
                let used_ram = sys.used_memory();
                let free_ram = sys.free_memory();
                let ram_pct = if total_ram > 0 {
                    (used_ram as f32 / total_ram as f32) * 100.0
                } else {
                    0.0
                };
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                HostMetricsTick {
                    cpu_usage: global_cpu_usage,
                    ram_used_bytes: used_ram,
                    ram_total_bytes: total_ram,
                    ram_free_bytes: free_ram,
                    ram_percentage: ram_pct,
                    gpu_usage: None,
                    timestamp: ts,
                }
            };

            if let Ok(json) = serde_json::to_string(&tick) {
                yield Ok(Event::default().data(json));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
