use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    pub host: String,
    pub port: u16,
    pub hardware: String,
    pub models_dir: String,
    pub runtimes_dir: String,
    #[serde(default = "default_true")]
    pub minimize_on_start: bool,
    #[serde(default = "default_true")]
    pub autostart_server: bool,
    #[serde(default = "default_true")]
    pub autostart_app: bool,
}

impl Default for GuiConfig {
    fn default() -> Self {
        let default_models = dirs::data_local_dir()
            .map(|p| p.join("monolai").join("models"))
            .or_else(|| dirs::home_dir().map(|h| h.join("models")))
            .unwrap_or_else(|| PathBuf::from("models"))
            .to_string_lossy()
            .to_string();

        let default_runtimes = dirs::data_local_dir()
            .map(|p| p.join("monolai").join("runtimes"))
            .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("share").join("monolai").join("runtimes")))
            .unwrap_or_else(|| PathBuf::from("runtimes"))
            .to_string_lossy()
            .to_string();

        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            hardware: "auto".to_string(),
            models_dir: default_models,
            runtimes_dir: default_runtimes,
            minimize_on_start: true,
            autostart_server: true,
            autostart_app: true,
        }
    }
}

impl GuiConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("monolai")
            .join("gui_config.json")
    }

    pub fn backend_yaml_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("monolai")
            .join("config.yaml")
    }

    pub fn load() -> Self {
        let mut cfg = Self::default();

        // 1. Load GUI preferences from gui_config.json if present
        let path = Self::config_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(gui_cfg) = serde_json::from_str::<Self>(&contents) {
                cfg = gui_cfg;
            }
        }

        // 2. config.yaml is the SINGLE SOURCE OF TRUTH for models, runtimes, hardware, host, and port!
        let yaml_path = Self::backend_yaml_path();
        if let Ok(contents) = fs::read_to_string(&yaml_path) {
            #[derive(Deserialize)]
            struct YamlPart {
                models: Option<String>,
                runtimes: Option<String>,
                hardware: Option<String>,
                host: Option<String>,
                port: Option<u16>,
            }
            if let Ok(y) = serde_yaml::from_str::<YamlPart>(&contents) {
                if let Some(m) = y.models {
                    if !m.trim().is_empty() {
                        cfg.models_dir = m;
                    }
                }
                if let Some(r) = y.runtimes {
                    if !r.trim().is_empty() {
                        cfg.runtimes_dir = r;
                    }
                }
                if let Some(h) = y.hardware {
                    if !h.trim().is_empty() {
                        cfg.hardware = h;
                    }
                }
                if let Some(host) = y.host {
                    if !host.trim().is_empty() {
                        cfg.host = host;
                    }
                }
                if let Some(port) = y.port {
                    if port > 0 {
                        cfg.port = port;
                    }
                }
            }
        }

        cfg
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write config file: {}", e))?;

        // Also sync to config.yaml preserving existing comments and formatting
        let _ = self.save_backend_yaml();

        Ok(())
    }

    pub fn save_backend_yaml(&self) -> Result<(), String> {
        let yaml_path = Self::backend_yaml_path();
        if let Some(parent) = yaml_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let final_yaml = if yaml_path.exists() {
            if let Ok(existing) = fs::read_to_string(&yaml_path) {
                let mut updated = update_yaml_field(&existing, "models", self.models_dir.trim());
                updated = update_yaml_field(&updated, "runtimes", self.runtimes_dir.trim());
                updated = update_yaml_field(&updated, "hardware", self.hardware.trim());
                let host_val = if self.host.trim().is_empty() { "0.0.0.0" } else { self.host.trim() };
                updated = update_yaml_field(&updated, "host", host_val);
                update_yaml_field(&updated, "port", &self.port.to_string())
            } else {
                format!(
                    "# Monolai Configuration File\nmodels: {}\nruntimes: {}\nhardware: {}\nhost: {}\nport: {}\n",
                    self.models_dir.trim(),
                    self.runtimes_dir.trim(),
                    self.hardware.trim(),
                    self.host.trim(),
                    self.port
                )
            }
        } else {
            format!(
                "# ==============================================================================\n\
                 # Monolai Configuration File\n\
                 # ==============================================================================\n\
                 # Target file path: {}\n#\n\
                 # 1. Models Directory:\n\
                 # Directory for storing local AI model files (.gguf, .safetensors).\n\
                 models: {}\n\n\
                 # 2. Runtimes Directory:\n\
                 # Directory where upstream runtime engines (llama-cpp, sd-cpp) are installed and managed.\n\
                 runtimes: {}\n\n\
                 # 3. Hardware Acceleration:\n\
                 # Preferred target acceleration: auto, cpu, cuda, rocm, vulkan, oneapi\n\
                 hardware: {}\n\n\
                 # 4. Network Configuration:\n\
                 # Bind address and HTTP port for the Monolai server\n\
                 host: {}\n\
                 port: {}\n",
                yaml_path.display(),
                self.models_dir.trim(),
                self.runtimes_dir.trim(),
                self.hardware.trim(),
                self.host.trim(),
                self.port
            )
        };

        fs::write(&yaml_path, final_yaml)
            .map_err(|e| format!("Failed to write config.yaml: {}", e))?;
        Ok(())
    }
}

/// Updates a single key: value in YAML content while preserving comments and layout.
pub fn update_yaml_field(content: &str, key: &str, value: &str) -> String {
    let mut replaced = false;
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    // 1. Try to find active key: e.g. "models: ..." or "  models : ..."
    for line in lines.iter_mut() {
        let trimmed = line.trim();
        if trimmed.starts_with(key) {
            let rest = trimmed[key.len()..].trim_start();
            if rest.starts_with(':') {
                let indent_len = line.len() - line.trim_start().len();
                let indent = &line[..indent_len];
                let val_part = rest[1..].trim();
                if let Some(comment_idx) = val_part.find('#') {
                    let comment = &val_part[comment_idx..];
                    *line = format!("{}{}: {} {}", indent, key, value, comment);
                } else {
                    *line = format!("{}{}: {}", indent, key, value);
                }
                replaced = true;
                break;
            }
        }
    }

    // 2. If not found as active key, check if it was commented out: e.g. "# hardware: auto" or "#hardware: auto"
    if !replaced {
        for line in lines.iter_mut() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let unhash = trimmed[1..].trim_start();
                if unhash.starts_with(key) {
                    let rest = unhash[key.len()..].trim_start();
                    if rest.starts_with(':') {
                        let indent_len = line.len() - line.trim_start().len();
                        let indent = &line[..indent_len];
                        *line = format!("{}{}: {}", indent, key, value);
                        replaced = true;
                        break;
                    }
                }
            }
        }
    }

    let mut result = lines.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }

    // 3. If key doesn't exist anywhere in the file, append it
    if !replaced {
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&format!("{}: {}\n", key, value));
    }

    result
}

pub fn is_autostart_app_enabled() -> bool {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Ok(auto) = auto_launch::AutoLaunchBuilder::new()
            .set_app_name("Monolai")
            .set_app_path(&exe_path.to_string_lossy())
            .build()
        {
            return auto.is_enabled().unwrap_or(false);
        }
    }
    false
}

pub fn set_autostart_app(enable: bool) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    let auto = auto_launch::AutoLaunchBuilder::new()
        .set_app_name("Monolai")
        .set_app_path(&exe_path.to_string_lossy())
        .build()
        .map_err(|e| format!("Failed to build autostart config: {}", e))?;

    if enable {
        let _ = auto.enable();
    } else {
        let _ = auto.disable();
    }
    Ok(())
}
