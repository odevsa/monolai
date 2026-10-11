use std::path::Path;

/// Locates temperature for integrated or attached GPU via hwmon
pub fn find_integrated_gpu_temp(dev_path: &Path) -> Option<f32> {
    // 1. Direct hwmon under device
    if let Ok(hwmon_entries) = std::fs::read_dir(dev_path.join("hwmon")) {
        for h in hwmon_entries.filter_map(|e| e.ok()) {
            let temp_input = h.path().join("temp1_input");
            if let Ok(val_str) = std::fs::read_to_string(temp_input) {
                if let Ok(m_deg) = val_str.trim().parse::<f32>() {
                    if m_deg > 0.0 && m_deg < 150000.0 {
                        return Some(m_deg / 1000.0);
                    }
                }
            }
        }
    }

    // 2. Fallback to package/die temperature from coretemp/k10temp
    if let Ok(hwmon_entries) = std::fs::read_dir("/sys/class/hwmon") {
        for h in hwmon_entries.filter_map(|e| e.ok()) {
            let name_path = h.path().join("name");
            let name = std::fs::read_to_string(name_path)
                .unwrap_or_default()
                .trim()
                .to_lowercase();
            if name.contains("coretemp")
                || name.contains("k10temp")
                || name.contains("zenpower")
                || name.contains("acpitz")
                || name.contains("thinkpad")
            {
                for i in 1..=8 {
                    let temp_input = h.path().join(format!("temp{}_input", i));
                    if let Ok(val_str) = std::fs::read_to_string(temp_input) {
                        if let Ok(m_deg) = val_str.trim().parse::<f32>() {
                            if m_deg > 0.0 && m_deg < 150000.0 {
                                return Some(m_deg / 1000.0);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
