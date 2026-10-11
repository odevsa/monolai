use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub memory_total_bytes: Option<u64>,
    pub driver_version: Option<String>,
    pub is_dedicated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GpuDeviceInfo {
    pub index: usize,
    pub name: String,
    pub vendor: String,
    pub memory_total_bytes: Option<u64>,
    pub memory_used_bytes: Option<u64>,
    pub memory_free_bytes: Option<u64>,
    pub driver_version: Option<String>,
    pub is_dedicated: bool,
    pub usage: Option<f32>,
    pub temperature_c: Option<f32>,
    pub clock_mhz: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HardwareReport {
    pub os: String,
    pub arch: String,
    pub available_accelerations: Vec<String>,
    pub recommended_acceleration: String,
    pub detected_gpus: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GpuUsageStats {
    pub gpu_usage: Option<f32>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
    pub vram_free_bytes: Option<u64>,
    pub vram_percentage: Option<f32>,
    pub temperature_c: Option<f32>,
    pub clock_mhz: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CpuInfo {
    pub usage: f32,
    pub cores: usize,
    pub brand: String,
    pub frequency_mhz: u64,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RamInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct OsInfo {
    pub name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub hostname: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct VramInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DiskItemInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StorageInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub percentage: f32,
    pub disks: Vec<DiskItemInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SysInfoResponse {
    pub cpu: CpuInfo,
    pub ram: RamInfo,
    pub os: OsInfo,
    pub storage: StorageInfo,
    #[schema(value_type = Option<Object>)]
    pub gpu: Option<GpuInfo>,
    pub vram: Option<VramInfo>,
    pub gpus: Vec<GpuDeviceInfo>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HostMetricsTick {
    pub cpu_usage: f32,
    pub cpu_temp_c: Option<f32>,
    pub cpu_frequency_mhz: Option<u64>,
    pub ram_used_bytes: u64,
    pub ram_total_bytes: u64,
    pub ram_free_bytes: u64,
    pub ram_percentage: f32,
    pub storage_used_bytes: u64,
    pub storage_total_bytes: u64,
    pub storage_percentage: f32,
    pub disk_read_bytes_sec: u64,
    pub disk_write_bytes_sec: u64,
    pub net_rx_bytes_sec: u64,
    pub net_tx_bytes_sec: u64,
    pub gpu_usage: Option<f32>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
    pub vram_free_bytes: Option<u64>,
    pub vram_percentage: Option<f32>,
    pub gpus: Vec<GpuDeviceInfo>,
    pub timestamp: u64,
}
