use crate::domain::{GpuDeviceInfo, GpuInfo, GpuUsageStats, HardwareReport};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::macos::detect_macos_gpus;
use super::nvidia::{find_nvidia_smi, query_nvidia_metrics, query_nvidia_smi_gpus};
use super::windows::detect_windows_display_controllers;

#[cfg(target_os = "linux")]
use super::linux::{detect_linux_gpus, query_linux_drm_metrics};

pub struct GpuTracker {
    pub gpus: Vec<GpuInfo>,
    pub primary_gpu: Option<GpuInfo>,
    usage_cache: Mutex<(Vec<GpuDeviceInfo>, Instant)>,
    intel_rc6_cache: Mutex<HashMap<String, (u64, Instant)>>,
    drm_fdinfo_cache: Mutex<HashMap<String, (HashMap<String, HashMap<String, u64>>, Instant)>>,
    nvidia_smi_path: Option<PathBuf>,
}

impl GpuTracker {
    pub fn new() -> Self {
        let mut gpus = Vec::new();
        let nvidia_smi_path = find_nvidia_smi();

        // 1. Query NVIDIA GPUs via nvidia-smi
        if let Some(ref smi) = nvidia_smi_path {
            gpus.extend(query_nvidia_smi_gpus(smi));
        }

        // 2. Query Windows display controllers
        gpus.extend(detect_windows_display_controllers());

        // 3. Query macOS display controllers
        gpus.extend(detect_macos_gpus());

        // 4. Query Linux PCI display controllers
        #[cfg(target_os = "linux")]
        {
            gpus.extend(detect_linux_gpus());

            // Check ROCm / KFD
            if Path::new("/dev/kfd").exists() && !gpus.iter().any(|g| g.vendor == "AMD") {
                gpus.push(GpuInfo {
                    name: "AMD ROCm GPU".to_string(),
                    vendor: "AMD".to_string(),
                    memory_total_bytes: None,
                    driver_version: None,
                    is_dedicated: true,
                });
            }
        }

        // Sort primary candidate: dedicated first, then largest memory
        let mut sorted = gpus.clone();
        sorted.sort_by(|a, b| {
            b.is_dedicated.cmp(&a.is_dedicated).then_with(|| {
                b.memory_total_bytes
                    .unwrap_or(0)
                    .cmp(&a.memory_total_bytes.unwrap_or(0))
            })
        });

        let primary_gpu = sorted.into_iter().next();

        Self {
            gpus,
            primary_gpu,
            usage_cache: Mutex::new((Vec::new(), Instant::now() - Duration::from_secs(10))),
            intel_rc6_cache: Mutex::new(HashMap::new()),
            drm_fdinfo_cache: Mutex::new(HashMap::new()),
            nvidia_smi_path,
        }
    }

    pub fn primary_gpu(&self) -> Option<GpuInfo> {
        self.primary_gpu.clone()
    }

    /// Queries real-time performance metrics (utilization, memory, temp, clock) for ALL detected GPUs.
    pub fn get_all_gpus(&self) -> Vec<GpuDeviceInfo> {
        let mut cache = self.usage_cache.lock().unwrap();
        if cache.1.elapsed() < Duration::from_millis(1000) && !cache.0.is_empty() {
            return cache.0.clone();
        }

        let mut results = Vec::new();
        let mut nvidia_covered_names = Vec::new();

        // 1. Query NVIDIA GPUs via nvidia-smi
        if let Some(ref smi) = self.nvidia_smi_path {
            let n_results = query_nvidia_metrics(smi);
            for item in n_results {
                nvidia_covered_names.push(item.name.clone());
                results.push(item);
            }
        }

        // 2. Query Linux DRM controllers (AMD, Intel, etc.)
        #[cfg(target_os = "linux")]
        {
            let linux_results = query_linux_drm_metrics(
                &self.gpus,
                &nvidia_covered_names,
                &self.drm_fdinfo_cache,
                &self.intel_rc6_cache,
            );
            results.extend(linux_results);
        }

        // 3. Fallback / Merge remaining static GPUs that had no dynamic sensors
        for g in &self.gpus {
            if !results.iter().any(|r| {
                r.name.eq_ignore_ascii_case(&g.name)
                    || (r.vendor == g.vendor && r.name.contains(&g.name))
            }) {
                results.push(GpuDeviceInfo {
                    index: results.len(),
                    name: g.name.clone(),
                    vendor: g.vendor.clone(),
                    memory_total_bytes: g.memory_total_bytes,
                    memory_used_bytes: None,
                    memory_free_bytes: g.memory_total_bytes,
                    driver_version: g.driver_version.clone(),
                    is_dedicated: g.is_dedicated,
                    usage: if g.is_dedicated { Some(0.0) } else { None },
                    temperature_c: None,
                    clock_mhz: None,
                });
            }
        }

        // Re-index sequentially
        for (i, item) in results.iter_mut().enumerate() {
            item.index = i;
        }

        *cache = (results.clone(), Instant::now());
        results
    }

    /// Legacy single-GPU usage stats for backwards-compatible API consumers.
    pub fn current_stats(&self) -> GpuUsageStats {
        let all = self.get_all_gpus();
        let primary = all.iter().find(|g| g.is_dedicated).or_else(|| all.first());

        match primary {
            Some(g) => {
                let vram_pct = match (g.memory_used_bytes, g.memory_total_bytes) {
                    (Some(u), Some(t)) if t > 0 => Some((u as f32 / t as f32) * 100.0),
                    _ => None,
                };
                GpuUsageStats {
                    gpu_usage: g.usage,
                    vram_used_bytes: g.memory_used_bytes,
                    vram_total_bytes: g.memory_total_bytes,
                    vram_free_bytes: g.memory_free_bytes,
                    vram_percentage: vram_pct,
                    temperature_c: g.temperature_c,
                    clock_mhz: g.clock_mhz,
                }
            }
            None => GpuUsageStats::default(),
        }
    }
}

static CACHED_HARDWARE_REPORT: OnceLock<HardwareReport> = OnceLock::new();

pub fn detect_hardware() -> HardwareReport {
    CACHED_HARDWARE_REPORT
        .get_or_init(detect_hardware_internal)
        .clone()
}

fn detect_hardware_internal() -> HardwareReport {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();

    let tracker = GpuTracker::new();
    let mut available = vec!["cpu".to_string()];
    let mut detected_gpus = Vec::new();

    let mut has_nvidia = false;
    let mut has_amd = false;
    let mut has_vulkan = false;

    for gpu in &tracker.gpus {
        let label = if let Some(mem) = gpu.memory_total_bytes {
            let mib = mem / (1024 * 1024);
            format!("{} ({} MiB)", gpu.name, mib)
        } else {
            gpu.name.clone()
        };
        detected_gpus.push(label);

        match gpu.vendor.as_str() {
            "NVIDIA" => has_nvidia = true,
            "AMD" => has_amd = true,
            _ => {}
        }
    }

    #[cfg(target_os = "linux")]
    {
        if Path::new("/dev/dri").exists() {
            has_vulkan = true;
        }
    }
    #[cfg(windows)]
    {
        has_vulkan = true;
    }

    if has_nvidia {
        available.push("cuda".to_string());
    }
    if has_amd {
        available.push("rocm".to_string());
    }

    let recommended = if has_nvidia {
        "cuda".to_string()
    } else if has_amd {
        "rocm".to_string()
    } else if os == "macos" {
        "metal".to_string()
    } else if has_vulkan {
        "vulkan".to_string()
    } else {
        "cpu".to_string()
    };

    HardwareReport {
        os,
        arch,
        available_accelerations: available,
        recommended_acceleration: recommended,
        detected_gpus,
    }
}

pub fn resolve_target_acceleration(configured_hw: Option<&str>) -> String {
    let report = detect_hardware();
    match configured_hw {
        Some(s) if !s.trim().is_empty() && s.trim() != "auto" => s.trim().to_string(),
        _ => report.recommended_acceleration,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_tracker_get_all_gpus() {
        let tracker = GpuTracker::new();
        // Warm up DRM fdinfo/rc6 cache
        let _ = tracker.get_all_gpus();
        std::thread::sleep(Duration::from_millis(1100));
        let gpus = tracker.get_all_gpus();

        for gpu in &gpus {
            println!(
                "Detected GPU #{}: name='{}', vendor='{}', usage={:?}%, temp={:?}C, clock={:?}MHz",
                gpu.index, gpu.name, gpu.vendor, gpu.usage, gpu.temperature_c, gpu.clock_mhz
            );
            if let Some(usage) = gpu.usage {
                assert!(
                    (0.0..=100.0).contains(&usage),
                    "GPU usage must be between 0 and 100%"
                );
            }
        }
    }
}
