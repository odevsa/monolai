pub mod drm;
pub mod hwmon;
pub mod sysfs;

use crate::domain::{GpuDeviceInfo, GpuInfo};
use drm::{calculate_drm_fdinfo_usage, DrmFdinfoCache};
use hwmon::find_integrated_gpu_temp;
use std::path::PathBuf;
use sysfs::{
    calculate_intel_rc6_usage, detect_linux_pci_gpus, find_intel_max_freq, find_intel_rc6_path,
    read_intel_act_freq, Rc6Cache,
};

pub fn detect_linux_gpus() -> Vec<GpuInfo> {
    detect_linux_pci_gpus()
}

pub fn query_linux_drm_metrics(
    static_gpus: &[GpuInfo],
    nvidia_covered_names: &[String],
    fdinfo_cache: &DrmFdinfoCache,
    rc6_cache: &Rc6Cache,
) -> Vec<GpuDeviceInfo> {
    let mut results = Vec::new();

    let entries = match std::fs::read_dir("/sys/class/drm") {
        Ok(e) => e,
        Err(_) => return results,
    };

    let mut card_dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            name.starts_with("card") && !name.contains('-')
        })
        .collect();
    card_dirs.sort();

    for card_path in card_dirs {
        let dev_path = card_path.join("device");
        if !dev_path.exists() {
            continue;
        }

        let vendor_hex = std::fs::read_to_string(dev_path.join("vendor"))
            .unwrap_or_default()
            .trim()
            .to_lowercase();

        if vendor_hex == "0x10de" && !nvidia_covered_names.is_empty() {
            // Already handled by nvidia-smi
            continue;
        }

        let is_amd = vendor_hex == "0x1002";
        let is_intel = vendor_hex == "0x8086";

        let vendor = if is_amd {
            "AMD".to_string()
        } else if is_intel {
            "Intel".to_string()
        } else {
            "Unknown".to_string()
        };

        let is_dedicated = is_amd;

        // Locate temperature from attached hwmon or fallback to integrated die temp
        let mut temp_c = None;
        if let Ok(hwmon_entries) = std::fs::read_dir(dev_path.join("hwmon")) {
            for h in hwmon_entries.filter_map(|e| e.ok()) {
                let temp_input = h.path().join("temp1_input");
                if let Ok(val_str) = std::fs::read_to_string(temp_input) {
                    if let Ok(m_deg) = val_str.trim().parse::<f32>() {
                        temp_c = Some(m_deg / 1000.0);
                        break;
                    }
                }
            }
        }
        if temp_c.is_none() && is_intel {
            temp_c = find_integrated_gpu_temp(&dev_path);
        }

        // Intel frequency / clock
        let mut clock_mhz = None;
        if is_intel {
            clock_mhz = read_intel_act_freq(&card_path);
        }

        // GPU usage and VRAM
        let mut mem_total = None;
        let mut mem_used = None;
        let mut gpu_usage = None;

        if is_amd {
            let total_p = dev_path.join("mem_info_vram_total");
            let used_p = dev_path.join("mem_info_vram_used");
            let busy_p = dev_path.join("gpu_busy_percent");

            mem_total = std::fs::read_to_string(total_p)
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok());
            mem_used = std::fs::read_to_string(used_p)
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok());
            gpu_usage = std::fs::read_to_string(busy_p)
                .ok()
                .and_then(|s| s.trim().parse::<f32>().ok());
        } else if is_intel {
            let card_name_str = card_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // 1. Query accurate hardware engine utilization via Linux DRM fdinfo (nanosecond precision)
            let (fdinfo_usage, fdinfo_mem) =
                calculate_drm_fdinfo_usage(&card_path, &dev_path, fdinfo_cache);
            gpu_usage = fdinfo_usage;
            if mem_used.is_none() {
                mem_used = fdinfo_mem;
            }

            // 2. Fallback to frequency-scaled RC6 if DRM fdinfo is not ready/available
            if gpu_usage.is_none() {
                if let Some(rc6_path) = find_intel_rc6_path(&card_path) {
                    let max_freq = find_intel_max_freq(&card_path);
                    gpu_usage = calculate_intel_rc6_usage(
                        &card_name_str,
                        &rc6_path,
                        clock_mhz,
                        max_freq,
                        rc6_cache,
                    );
                }
            }
        }

        let mem_free = match (mem_total, mem_used) {
            (Some(t), Some(u)) => Some(t.saturating_sub(u)),
            _ => None,
        };

        // Find corresponding name from detected static list
        let matched_info = static_gpus.iter().find(|g| g.vendor == vendor);
        let name = matched_info
            .map(|g| g.name.clone())
            .unwrap_or_else(|| format!("{} Graphics Controller", vendor));

        // Avoid duplicate if already inserted
        if !results.iter().any(|r| r.name == name) {
            results.push(GpuDeviceInfo {
                index: results.len(),
                name,
                vendor,
                memory_total_bytes: mem_total.or(matched_info.and_then(|g| g.memory_total_bytes)),
                memory_used_bytes: mem_used,
                memory_free_bytes: mem_free,
                driver_version: matched_info.and_then(|g| g.driver_version.clone()),
                is_dedicated,
                usage: gpu_usage,
                temperature_c: temp_c,
                clock_mhz,
            });
        }
    }

    results
}
