use crate::domain::{
    CpuInfo, HostMetricsTick, OsInfo, RamInfo, SysInfoResponse, VramInfo,
};
use crate::infrastructure::hardware::GpuTracker;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

#[derive(Clone)]
pub struct HostService {
    sys: Arc<Mutex<System>>,
    gpu_tracker: Arc<GpuTracker>,
}

impl HostService {
    pub fn new(sys: Arc<Mutex<System>>, gpu_tracker: Arc<GpuTracker>) -> Self {
        Self { sys, gpu_tracker }
    }

    pub fn get_sysinfo(&self) -> SysInfoResponse {
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_cpu();
        sys.refresh_memory();

        let cpus = sys.cpus();
        let cpus_len = cpus.len();
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

        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let kernel_ver = System::kernel_version().unwrap_or_else(|| "".to_string());
        let os_ver = System::os_version().unwrap_or_else(|| "".to_string());
        let hostname = System::host_name().unwrap_or_else(|| "localhost".to_string());
        let uptime = System::uptime();

        drop(sys);

        let gpu = self.gpu_tracker.primary_gpu();
        let gpu_stats = self.gpu_tracker.current_stats();
        let vram = if gpu.is_some() {
            Some(VramInfo {
                total_bytes: gpu_stats.vram_total_bytes.unwrap_or(0),
                used_bytes: gpu_stats.vram_used_bytes.unwrap_or(0),
                free_bytes: gpu_stats.vram_free_bytes.unwrap_or(0),
                percentage: gpu_stats.vram_percentage.unwrap_or(0.0),
            })
        } else {
            None
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        SysInfoResponse {
            cpu: CpuInfo {
                usage: global_cpu_usage,
                cores: cpus_len,
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
                name: os_name,
                kernel_version: kernel_ver,
                os_version: os_ver,
                hostname,
                uptime_seconds: uptime,
            },
            gpu,
            vram,
            timestamp,
        }
    }

    pub fn get_metrics_tick(&self) -> HostMetricsTick {
        let mut sys = self.sys.lock().unwrap();
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
        drop(sys);

        let gpu_stats = self.gpu_tracker.current_stats();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        HostMetricsTick {
            cpu_usage: global_cpu_usage,
            ram_used_bytes: used_ram,
            ram_total_bytes: total_ram,
            ram_free_bytes: free_ram,
            ram_percentage: ram_pct,
            gpu_usage: gpu_stats.gpu_usage,
            vram_used_bytes: gpu_stats.vram_used_bytes,
            vram_total_bytes: gpu_stats.vram_total_bytes,
            vram_free_bytes: gpu_stats.vram_free_bytes,
            vram_percentage: gpu_stats.vram_percentage,
            timestamp,
        }
    }
}
