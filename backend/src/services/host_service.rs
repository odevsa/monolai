use crate::domain::{
    CpuInfo, DiskItemInfo, GpuDeviceInfo, HostMetricsTick, OsInfo, RamInfo, StorageInfo,
    SysInfoResponse, VramInfo,
};
use crate::infrastructure::hardware::GpuTracker;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{Components, Disks, Networks, System};

struct IoState {
    last_tick_time: Instant,
    last_net_rx_total: u64,
    last_net_tx_total: u64,
    last_disk_read_sectors: u64,
    last_disk_write_sectors: u64,
    cached_tick: Option<HostMetricsTick>,
}

#[derive(Clone)]
pub struct HostService {
    sys: Arc<Mutex<System>>,
    gpu_tracker: Arc<GpuTracker>,
    disks: Arc<Mutex<Disks>>,
    networks: Arc<Mutex<Networks>>,
    components: Arc<Mutex<Components>>,
    io_state: Arc<Mutex<IoState>>,
}

impl HostService {
    pub fn new(sys: Arc<Mutex<System>>, gpu_tracker: Arc<GpuTracker>) -> Self {
        let disks = Arc::new(Mutex::new(Disks::new_with_refreshed_list()));
        let networks = Arc::new(Mutex::new(Networks::new_with_refreshed_list()));
        let components = Arc::new(Mutex::new(Components::new_with_refreshed_list()));

        let mut initial_rx = 0;
        let mut initial_tx = 0;
        if let Ok(net) = networks.lock() {
            for (_name, data) in net.iter() {
                initial_rx += data.total_received();
                initial_tx += data.total_transmitted();
            }
        }

        let (initial_read_sec, initial_write_sec) = Self::query_disk_io_sectors();

        let io_state = Arc::new(Mutex::new(IoState {
            last_tick_time: Instant::now() - Duration::from_secs(2),
            last_net_rx_total: initial_rx,
            last_net_tx_total: initial_tx,
            last_disk_read_sectors: initial_read_sec,
            last_disk_write_sectors: initial_write_sec,
            cached_tick: None,
        }));

        Self {
            sys,
            gpu_tracker,
            disks,
            networks,
            components,
            io_state,
        }
    }

    fn query_cpu_temperature(components: &Components) -> Option<f32> {
        let mut best_temp: Option<f32> = None;
        for c in components.iter() {
            let label_lower = c.label().to_lowercase();
            if label_lower.contains("core")
                || label_lower.contains("cpu")
                || label_lower.contains("tctl")
                || label_lower.contains("tdie")
                || label_lower.contains("package")
                || label_lower.contains("k10temp")
            {
                let temp = c.temperature();
                if temp > 0.0 && temp < 130.0 {
                    best_temp = Some(best_temp.map_or(temp, |b| b.max(temp)));
                }
            }
        }

        if best_temp.is_some() {
            return best_temp;
        }

        // Linux hwmon fallback
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    let name = std::fs::read_to_string(path.join("name"))
                        .unwrap_or_default()
                        .to_lowercase();
                    if name.contains("coretemp") || name.contains("k10temp") || name.contains("cpu") {
                        if let Ok(val) = std::fs::read_to_string(path.join("temp1_input")) {
                            if let Ok(mdeg) = val.trim().parse::<f32>() {
                                let deg = mdeg / 1000.0;
                                if deg > 0.0 && deg < 130.0 {
                                    return Some(deg);
                                }
                            }
                        }
                    }
                }
            }

            if let Ok(val) = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
                if let Ok(mdeg) = val.trim().parse::<f32>() {
                    let deg = mdeg / 1000.0;
                    if deg > 0.0 && deg < 130.0 {
                        return Some(deg);
                    }
                }
            }
        }

        None
    }

    fn query_disk_io_sectors() -> (u64, u64) {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/diskstats") {
                let mut total_read_sectors = 0u64;
                let mut total_write_sectors = 0u64;

                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 10 {
                        let dev_name = parts[2];
                        if dev_name.starts_with("loop") || dev_name.starts_with("ram") {
                            continue;
                        }

                        // Match base block devices (sda, sdb, nvme0n1, vda, mmcblk0)
                        let is_base_dev = if dev_name.starts_with("sd") || dev_name.starts_with("vd") || dev_name.starts_with("hd") {
                            dev_name.len() == 3 && dev_name.chars().nth(2).map_or(false, |c| c.is_ascii_alphabetic())
                        } else if dev_name.starts_with("nvme") {
                            dev_name.ends_with("n1") && !dev_name.contains('p')
                        } else if dev_name.starts_with("mmcblk") {
                            !dev_name.contains('p')
                        } else {
                            false
                        };

                        if is_base_dev {
                            let read_sec = parts[5].parse::<u64>().unwrap_or(0);
                            let write_sec = parts[9].parse::<u64>().unwrap_or(0);
                            total_read_sectors += read_sec;
                            total_write_sectors += write_sec;
                        }
                    }
                }

                if total_read_sectors > 0 || total_write_sectors > 0 {
                    return (total_read_sectors, total_write_sectors);
                }
            }
        }

        (0, 0)
    }

    fn collect_storage_info(disks: &mut Disks) -> StorageInfo {
        disks.refresh();
        let mut disk_items = Vec::new();
        let mut seen_physical_devices = HashSet::new();
        let mut total_bytes = 0u64;
        let mut available_bytes = 0u64;

        for disk in disks.iter() {
            let total = disk.total_space();
            let avail = disk.available_space();
            let used = total.saturating_sub(avail);
            let pct = if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            let name = disk.name().to_string_lossy().to_string();
            let mount = disk.mount_point().to_string_lossy().to_string();

            // Skip pseudo/ephemeral filesystems
            if mount.starts_with("/snap") || name.contains("loop") {
                continue;
            }

            // Deduplicate devices so multiple subvolumes (e.g. btrfs /home, /var, / on /dev/sda2)
            // or bind mounts are NOT summed multiple times for total disk capacity
            let device_key = if !name.is_empty() { name.clone() } else { mount.clone() };
            if seen_physical_devices.insert(device_key) {
                total_bytes += total;
                available_bytes += avail;
            }

            disk_items.push(DiskItemInfo {
                name: if name.is_empty() { mount.clone() } else { name },
                mount_point: mount,
                total_bytes: total,
                used_bytes: used,
                free_bytes: avail,
                percentage: pct,
            });
        }

        let used_bytes = total_bytes.saturating_sub(available_bytes);
        let percentage = if total_bytes > 0 {
            (used_bytes as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };

        StorageInfo {
            total_bytes,
            used_bytes,
            free_bytes: available_bytes,
            percentage,
            disks: disk_items,
        }
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

        let mut comp = self.components.lock().unwrap();
        comp.refresh();
        let cpu_temp = Self::query_cpu_temperature(&comp);
        drop(comp);

        let mut disks = self.disks.lock().unwrap();
        let storage = Self::collect_storage_info(&mut disks);
        drop(disks);

        let all_gpus: Vec<GpuDeviceInfo> = self.gpu_tracker.get_all_gpus();
        let primary_gpu = self.gpu_tracker.primary_gpu();
        let gpu_stats = self.gpu_tracker.current_stats();

        let vram = if primary_gpu.is_some() || !all_gpus.is_empty() {
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
                temperature_c: cpu_temp,
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
            storage,
            gpu: primary_gpu,
            vram,
            gpus: all_gpus,
            timestamp,
        }
    }

    pub fn get_metrics_tick(&self) -> HostMetricsTick {
        let mut io = self.io_state.lock().unwrap();
        if io.last_tick_time.elapsed() < Duration::from_millis(900) {
            if let Some(ref cached) = io.cached_tick {
                return cached.clone();
            }
        }

        let mut sys = self.sys.lock().unwrap();
        sys.refresh_cpu();
        sys.refresh_memory();

        let global_cpu_usage = sys.global_cpu_info().cpu_usage();
        let cpu_freq = sys.cpus().first().map(|c| c.frequency());
        let total_ram = sys.total_memory();
        let used_ram = sys.used_memory();
        let free_ram = sys.free_memory();
        let ram_pct = if total_ram > 0 {
            (used_ram as f32 / total_ram as f32) * 100.0
        } else {
            0.0
        };
        drop(sys);

        let mut comp = self.components.lock().unwrap();
        comp.refresh();
        let cpu_temp = Self::query_cpu_temperature(&comp);
        drop(comp);

        let mut disks = self.disks.lock().unwrap();
        let storage = Self::collect_storage_info(&mut disks);
        drop(disks);

        let mut networks = self.networks.lock().unwrap();
        networks.refresh();

        let mut total_rx = 0u64;
        let mut total_tx = 0u64;
        for (_name, data) in networks.iter() {
            total_rx += data.total_received();
            total_tx += data.total_transmitted();
        }
        drop(networks);

        let (curr_read_sec, curr_write_sec) = Self::query_disk_io_sectors();

        let elapsed_secs = io.last_tick_time.elapsed().as_secs_f32().max(0.5);

        // Network deltas
        let rx_delta = total_rx.saturating_sub(io.last_net_rx_total);
        let tx_delta = total_tx.saturating_sub(io.last_net_tx_total);
        let net_rx_bytes_sec = (rx_delta as f32 / elapsed_secs) as u64;
        let net_tx_bytes_sec = (tx_delta as f32 / elapsed_secs) as u64;

        // Disk I/O deltas (sectors are 512 bytes each)
        let disk_read_delta = curr_read_sec.saturating_sub(io.last_disk_read_sectors) * 512;
        let disk_write_delta = curr_write_sec.saturating_sub(io.last_disk_write_sectors) * 512;
        let disk_read_bytes_sec = (disk_read_delta as f32 / elapsed_secs) as u64;
        let disk_write_bytes_sec = (disk_write_delta as f32 / elapsed_secs) as u64;

        io.last_tick_time = Instant::now();
        io.last_net_rx_total = total_rx;
        io.last_net_tx_total = total_tx;
        io.last_disk_read_sectors = curr_read_sec;
        io.last_disk_write_sectors = curr_write_sec;

        let all_gpus = self.gpu_tracker.get_all_gpus();
        let gpu_stats = self.gpu_tracker.current_stats();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let tick = HostMetricsTick {
            cpu_usage: global_cpu_usage,
            cpu_temp_c: cpu_temp,
            cpu_frequency_mhz: cpu_freq,
            ram_used_bytes: used_ram,
            ram_total_bytes: total_ram,
            ram_free_bytes: free_ram,
            ram_percentage: ram_pct,
            storage_used_bytes: storage.used_bytes,
            storage_total_bytes: storage.total_bytes,
            storage_percentage: storage.percentage,
            disk_read_bytes_sec,
            disk_write_bytes_sec,
            net_rx_bytes_sec,
            net_tx_bytes_sec,
            gpu_usage: gpu_stats.gpu_usage,
            vram_used_bytes: gpu_stats.vram_used_bytes,
            vram_total_bytes: gpu_stats.vram_total_bytes,
            vram_free_bytes: gpu_stats.vram_free_bytes,
            vram_percentage: gpu_stats.vram_percentage,
            gpus: all_gpus,
            timestamp,
        };

        io.cached_tick = Some(tick.clone());
        tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::Disks;

    #[test]
    fn test_collect_storage_info() {
        let mut disks = Disks::new_with_refreshed_list();
        let storage = HostService::collect_storage_info(&mut disks);
        println!("STORAGE TOTAL: {} GB", storage.total_bytes as f64 / 1_000_000_000.0);
        for d in &storage.disks {
            println!("  DISK: name='{}', mount='{}', total={} GB", d.name, d.mount_point, d.total_bytes as f64 / 1_000_000_000.0);
        }
        assert!(storage.total_bytes < 300_000_000_000, "Storage should not duplicate partitions! Got {}", storage.total_bytes);
    }
}
