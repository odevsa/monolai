use crate::domain::GpuInfo;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;

pub type Rc6Cache = Mutex<HashMap<String, (u64, Instant)>>;

pub fn detect_linux_pci_gpus() -> Vec<GpuInfo> {
    let mut list = Vec::new();

    // 1. Prefer lspci for descriptive model names (e.g., "Intel UHD Graphics 620", "GeForce RTX 4080")
    if let Ok(output) = Command::new("lspci").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line_lower = line.to_lowercase();
                if line_lower.contains("vga compatible controller")
                    || line_lower.contains("3d controller")
                    || line_lower.contains("display controller")
                {
                    let is_nvidia = line_lower.contains("nvidia");
                    let is_amd = line_lower.contains("amd")
                        || line_lower.contains("radeon")
                        || line_lower.contains("advanced micro devices");
                    let is_intel = line_lower.contains("intel");

                    let vendor = if is_nvidia {
                        "NVIDIA".to_string()
                    } else if is_amd {
                        "AMD".to_string()
                    } else if is_intel {
                        "Intel".to_string()
                    } else {
                        "Unknown".to_string()
                    };

                    let is_dedicated = is_nvidia || (is_amd && !line_lower.contains("renoir"));

                    let name = if let Some(idx) = line.find(": ") {
                        let desc = &line[idx + 2..];
                        if let Some(sub_idx) = desc.find(": ") {
                            desc[sub_idx + 2..].trim().to_string()
                        } else {
                            desc.trim().to_string()
                        }
                    } else {
                        format!("{} Graphics Device", vendor)
                    };

                    let mut mem_total = None;
                    if is_amd {
                        let card_paths = [
                            "/sys/class/drm/card0/device/mem_info_vram_total",
                            "/sys/class/drm/card1/device/mem_info_vram_total",
                            "/sys/class/drm/card2/device/mem_info_vram_total",
                        ];
                        for p in card_paths {
                            if let Ok(content) = std::fs::read_to_string(p) {
                                if let Ok(bytes) = content.trim().parse::<u64>() {
                                    mem_total = Some(bytes);
                                    break;
                                }
                            }
                        }
                    }

                    list.push(GpuInfo {
                        name,
                        vendor,
                        memory_total_bytes: mem_total,
                        driver_version: None,
                        is_dedicated,
                    });
                }
            }
        }
    }

    if !list.is_empty() {
        return list;
    }

    // 2. Fallback: /sys/bus/pci/devices inspection
    if let Ok(entries) = std::fs::read_dir("/sys/bus/pci/devices") {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let class_path = path.join("class");
            if let Ok(class_str) = std::fs::read_to_string(&class_path) {
                let trimmed = class_str.trim();
                if trimmed.starts_with("0x03") {
                    let vendor_path = path.join("vendor");
                    let vendor_hex = std::fs::read_to_string(&vendor_path)
                        .unwrap_or_default()
                        .trim()
                        .to_lowercase();

                    let (vendor, is_dedicated, default_name) = match vendor_hex.as_str() {
                        "0x10de" => (
                            "NVIDIA".to_string(),
                            true,
                            "NVIDIA Graphics Controller".to_string(),
                        ),
                        "0x1002" => (
                            "AMD".to_string(),
                            true,
                            "AMD Radeon Graphics Controller".to_string(),
                        ),
                        "0x8086" => (
                            "Intel".to_string(),
                            false,
                            "Intel Graphics Controller".to_string(),
                        ),
                        _ => (
                            "Unknown".to_string(),
                            false,
                            "PCI Display Controller".to_string(),
                        ),
                    };

                    let mut mem_total = None;
                    if vendor == "AMD" {
                        let card_paths = [
                            "/sys/class/drm/card0/device/mem_info_vram_total",
                            "/sys/class/drm/card1/device/mem_info_vram_total",
                            "/sys/class/drm/card2/device/mem_info_vram_total",
                        ];
                        for p in card_paths {
                            if let Ok(content) = std::fs::read_to_string(p) {
                                if let Ok(bytes) = content.trim().parse::<u64>() {
                                    mem_total = Some(bytes);
                                    break;
                                }
                            }
                        }
                    }

                    list.push(GpuInfo {
                        name: default_name,
                        vendor,
                        memory_total_bytes: mem_total,
                        driver_version: None,
                        is_dedicated,
                    });
                }
            }
        }
    }

    list
}

pub fn find_intel_rc6_path(card_path: &Path) -> Option<PathBuf> {
    let card_name = card_path.file_name()?;
    let candidates = [
        card_path
            .join("device/drm")
            .join(card_name)
            .join("gt/gt0/rc6_residency_ms"),
        card_path
            .join("device/drm")
            .join(card_name)
            .join("gt/rc6_residency_ms"),
        card_path.join("gt/gt0/rc6_residency_ms"),
        card_path.join("gt/rc6_residency_ms"),
        card_path.join("power/rc6_residency_ms"),
    ];
    for p in candidates {
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn find_intel_max_freq(card_path: &Path) -> Option<u32> {
    let card_name = card_path.file_name()?;
    let candidates = [
        card_path
            .join("device/drm")
            .join(card_name)
            .join("gt_max_freq_mhz"),
        card_path
            .join("device/drm")
            .join(card_name)
            .join("gt_RP0_freq_mhz"),
        card_path.join("gt_max_freq_mhz"),
        card_path.join("gt_RP0_freq_mhz"),
    ];
    for p in &candidates {
        if let Ok(s) = std::fs::read_to_string(p) {
            if let Ok(val) = s.trim().parse::<u32>() {
                if val > 0 {
                    return Some(val);
                }
            }
        }
    }
    None
}

pub fn read_intel_act_freq(card_path: &Path) -> Option<u32> {
    let card_name = card_path.file_name()?;
    let freq_paths = [
        card_path
            .join("device/drm")
            .join(card_name)
            .join("gt_act_freq_mhz"),
        card_path
            .join("device/drm")
            .join(card_name)
            .join("gt_cur_freq_mhz"),
        card_path.join("gt_act_freq_mhz"),
        card_path.join("gt_cur_freq_mhz"),
    ];
    for p in &freq_paths {
        if let Ok(s) = std::fs::read_to_string(p) {
            if let Ok(val) = s.trim().parse::<u32>() {
                return Some(val);
            }
        }
    }
    None
}

pub fn calculate_intel_rc6_usage(
    card_key: &str,
    rc6_path: &Path,
    clock_mhz: Option<u32>,
    max_freq_mhz: Option<u32>,
    rc6_cache: &Rc6Cache,
) -> Option<f32> {
    let content = std::fs::read_to_string(rc6_path).ok()?;
    let curr_rc6 = content.trim().parse::<u64>().ok()?;
    let now = Instant::now();

    let mut cache = rc6_cache.lock().unwrap();
    let prev = cache.get(card_key).cloned();
    cache.insert(card_key.to_string(), (curr_rc6, now));

    if let Some((prev_rc6, prev_time)) = prev {
        let elapsed_ms = now.duration_since(prev_time).as_secs_f32() * 1000.0;
        if elapsed_ms > 200.0 && elapsed_ms < 10000.0 {
            let rc6_delta = curr_rc6.saturating_sub(prev_rc6) as f32;
            let busy_ms = (elapsed_ms - rc6_delta).max(0.0);
            let wake_pct = ((busy_ms / elapsed_ms) * 100.0).clamp(0.0, 100.0);

            // Scale wake percentage by operating frequency ratio to accurately estimate EU activity
            let max_f = max_freq_mhz.unwrap_or(1150).max(300) as f32;
            let cur_f = clock_mhz.unwrap_or(350).max(100) as f32;
            let freq_factor = (cur_f / max_f).clamp(0.15, 1.0);
            let usage_pct = (wake_pct * freq_factor).clamp(0.0, 100.0);

            return Some(usage_pct);
        }
    }
    None
}
