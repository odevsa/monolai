use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

pub type DrmFdinfoCache = Mutex<HashMap<String, (HashMap<String, HashMap<String, u64>>, Instant)>>;

pub fn calculate_drm_fdinfo_usage(
    card_path: &Path,
    dev_path: &Path,
    fdinfo_cache: &DrmFdinfoCache,
) -> (Option<f32>, Option<u64>) {
    let card_key = match card_path.file_name() {
        Some(k) => k.to_string_lossy().to_string(),
        None => return (None, None),
    };

    let target_pdev = dev_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let target_driver = std::fs::read_link(dev_path.join("driver"))
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "i915".to_string());

    let now = Instant::now();
    let mut curr_clients: HashMap<String, HashMap<String, u64>> = HashMap::new();
    let mut resident_mem: HashMap<String, u64> = HashMap::new();

    if let Ok(proc_entries) = std::fs::read_dir("/proc") {
        for proc_entry in proc_entries.flatten() {
            let pid_name = proc_entry.file_name();
            let pid_bytes = pid_name.as_encoded_bytes();
            if !pid_bytes.iter().all(|b| b.is_ascii_digit()) {
                continue;
            }

            let fd_dir = proc_entry.path().join("fd");
            if let Ok(fd_entries) = std::fs::read_dir(&fd_dir) {
                for fd_entry in fd_entries.flatten() {
                    let target = match std::fs::read_link(fd_entry.path()) {
                        Ok(t) => t,
                        Err(_) => continue,
                    };

                    if !target.to_string_lossy().contains("/dev/dri/") {
                        continue;
                    }

                    let fdinfo_path = proc_entry.path().join("fdinfo").join(fd_entry.file_name());
                    let content = match std::fs::read_to_string(&fdinfo_path) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    let mut client_id = None;
                    let mut matches_device = false;
                    let mut engines = HashMap::new();
                    let mut mem_kib = 0u64;

                    for line in content.lines() {
                        if let Some(rest) = line.strip_prefix("drm-client-id:") {
                            client_id = Some(rest.trim().to_string());
                        } else if let Some(rest) = line.strip_prefix("drm-pdev:") {
                            if rest.trim() == target_pdev {
                                matches_device = true;
                            }
                        } else if let Some(rest) = line.strip_prefix("drm-driver:") {
                            if rest.trim() == target_driver {
                                matches_device = true;
                            }
                        } else if let Some(rest) = line.strip_prefix("drm-engine-") {
                            if let Some((eng, val_part)) = rest.split_once(':') {
                                let eng_name = eng.trim();
                                let val_str =
                                    val_part.split_whitespace().next().unwrap_or_default();
                                if let Ok(val) = val_str.parse::<u64>() {
                                    engines.insert(eng_name.to_string(), val);
                                }
                            }
                        } else if let Some(rest) = line
                            .strip_prefix("drm-resident-system0:")
                            .or_else(|| line.strip_prefix("drm-resident-vram:"))
                            .or_else(|| line.strip_prefix("drm-total-system0:"))
                        {
                            let val_str = rest.split_whitespace().next().unwrap_or_default();
                            if let Ok(val) = val_str.parse::<u64>() {
                                mem_kib = mem_kib.max(val);
                            }
                        }
                    }

                    if matches_device || (target_pdev.is_empty() && target_driver.is_empty()) {
                        if let Some(cid) = client_id {
                            let entry = curr_clients.entry(cid.clone()).or_default();
                            for (eng, val) in engines {
                                let cur = entry.entry(eng).or_insert(0);
                                *cur = (*cur).max(val);
                            }
                            if mem_kib > 0 {
                                let mem_bytes = mem_kib * 1024;
                                let cur_mem = resident_mem.entry(cid).or_insert(0);
                                *cur_mem = (*cur_mem).max(mem_bytes);
                            }
                        }
                    }
                }
            }
        }
    }

    let total_vram_used = if !resident_mem.is_empty() {
        Some(resident_mem.values().copied().sum::<u64>())
    } else {
        None
    };

    if curr_clients.is_empty() {
        return (None, total_vram_used);
    }

    let mut cache = fdinfo_cache.lock().unwrap();
    let prev = cache.get(&card_key).cloned();
    cache.insert(card_key, (curr_clients.clone(), now));

    if let Some((prev_clients, prev_time)) = prev {
        let elapsed_ns = now.duration_since(prev_time).as_nanos() as u64;
        if elapsed_ns >= 300_000_000 && elapsed_ns <= 10_000_000_000 {
            let mut engine_deltas: HashMap<String, u64> = HashMap::new();
            for (cid, curr_engs) in &curr_clients {
                if let Some(prev_engs) = prev_clients.get(cid) {
                    for (eng, &curr_ns) in curr_engs {
                        if let Some(&prev_ns) = prev_engs.get(eng) {
                            let delta = curr_ns.saturating_sub(prev_ns);
                            *engine_deltas.entry(eng.clone()).or_insert(0) += delta;
                        }
                    }
                }
            }

            let max_busy_ns = engine_deltas.values().copied().max().unwrap_or(0);
            let usage_pct = ((max_busy_ns as f64 / elapsed_ns as f64) * 100.0) as f32;
            return (Some(usage_pct.clamp(0.0, 100.0)), total_vram_used);
        }
    }

    (None, total_vram_used)
}
