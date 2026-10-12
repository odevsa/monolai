use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const CONFIG_TEMPLATE: &str = include_str!("../../../config.example.yaml");

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuiPreferences {
    #[serde(default = "default_true")]
    pub minimize_on_start: bool,
    #[serde(default = "default_true")]
    pub autostart_server: bool,
    #[serde(default = "default_true")]
    pub autostart_app: bool,
}

impl Default for GuiPreferences {
    fn default() -> Self {
        Self {
            minimize_on_start: true,
            autostart_server: true,
            autostart_app: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    pub host: String,
    pub port: u16,
    pub models: String,
    pub runtimes: String,
    #[serde(default)]
    pub gui: GuiPreferences,
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
            .or_else(|| {
                dirs::home_dir().map(|h| {
                    h.join(".local")
                        .join("share")
                        .join("monolai")
                        .join("runtimes")
                })
            })
            .unwrap_or_else(|| PathBuf::from("runtimes"))
            .to_string_lossy()
            .to_string();

        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            models: default_models,
            runtimes: default_runtimes,
            gui: GuiPreferences::default(),
        }
    }
}

pub fn render_config_template(
    config_path: &str,
    models: &str,
    runtimes: &str,
    host: &str,
    port: u16,
    gui: &GuiPreferences,
) -> String {
    CONFIG_TEMPLATE
        .replace("{config_path}", config_path)
        .replace("{models}", models)
        .replace("{runtimes}", runtimes)
        .replace("{host}", host)
        .replace("{port}", &port.to_string())
        .replace("{minimize_on_start}", &gui.minimize_on_start.to_string())
        .replace("{autostart_server}", &gui.autostart_server.to_string())
        .replace("{autostart_app}", &gui.autostart_app.to_string())
}

impl GuiConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("monolai")
            .join("config.yaml")
    }

    pub fn load() -> Self {
        let mut cfg = Self::default();
        let yaml_path = Self::config_path();

        // 1. Single source of truth: config.yaml
        if let Ok(contents) = fs::read_to_string(&yaml_path) {
            #[derive(Deserialize)]
            struct YamlPart {
                models: Option<String>,
                runtimes: Option<String>,
                host: Option<String>,
                port: Option<u16>,
                gui: Option<GuiPreferences>,
            }

            let mut has_gui_in_yaml = false;

            if let Ok(y) = serde_yaml::from_str::<YamlPart>(&contents) {
                if let Some(m) = y.models {
                    if !m.trim().is_empty() {
                        cfg.models = m;
                    }
                }
                if let Some(r) = y.runtimes {
                    if !r.trim().is_empty() {
                        cfg.runtimes = r;
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
                if let Some(gui) = y.gui {
                    cfg.gui = gui;
                    has_gui_in_yaml = true;
                }
            }

            // If config.yaml exists but does not have the gui section, create it
            if !has_gui_in_yaml {
                let updated = update_or_append_gui_section(&contents, &cfg.gui);
                let _ = fs::write(&yaml_path, updated);
            }
        } else {
            // 2. Migration fallback: check legacy gui_config.json if config.yaml does not exist
            let legacy_json_path = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("monolai")
                .join("gui_config.json");
            if let Ok(contents) = fs::read_to_string(&legacy_json_path) {
                #[derive(Deserialize)]
                struct LegacyGuiConfig {
                    host: Option<String>,
                    port: Option<u16>,
                    models_dir: Option<String>,
                    runtimes_dir: Option<String>,
                    models: Option<String>,
                    runtimes: Option<String>,
                    minimize_on_start: Option<bool>,
                    autostart_server: Option<bool>,
                    autostart_app: Option<bool>,
                }
                if let Ok(legacy) = serde_json::from_str::<LegacyGuiConfig>(&contents) {
                    if let Some(m) = legacy.models.or(legacy.models_dir) {
                        if !m.trim().is_empty() {
                            cfg.models = m;
                        }
                    }
                    if let Some(r) = legacy.runtimes.or(legacy.runtimes_dir) {
                        if !r.trim().is_empty() {
                            cfg.runtimes = r;
                        }
                    }
                    if let Some(host) = legacy.host {
                        if !host.trim().is_empty() {
                            cfg.host = host;
                        }
                    }
                    if let Some(port) = legacy.port {
                        if port > 0 {
                            cfg.port = port;
                        }
                    }
                    if let Some(val) = legacy.minimize_on_start {
                        cfg.gui.minimize_on_start = val;
                    }
                    if let Some(val) = legacy.autostart_server {
                        cfg.gui.autostart_server = val;
                    }
                    if let Some(val) = legacy.autostart_app {
                        cfg.gui.autostart_app = val;
                    }
                }
                let _ = fs::remove_file(legacy_json_path);
            }

            // If config.yaml doesn't exist, create it with full template
            let initial = cfg.to_full_yaml_string(&yaml_path);
            if let Some(parent) = yaml_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&yaml_path, initial);
        }

        cfg
    }

    pub fn save(&self) -> Result<(), String> {
        let yaml_path = Self::config_path();
        if let Some(parent) = yaml_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let final_yaml = if yaml_path.exists() {
            if let Ok(existing) = fs::read_to_string(&yaml_path) {
                let mut updated = update_yaml_field(&existing, "models", self.models.trim());
                updated = update_yaml_field(&updated, "runtimes", self.runtimes.trim());
                let host_val = if self.host.trim().is_empty() {
                    "0.0.0.0"
                } else {
                    self.host.trim()
                };
                updated = update_yaml_field(&updated, "host", host_val);
                updated = update_yaml_field(&updated, "port", &self.port.to_string());
                update_or_append_gui_section(&updated, &self.gui)
            } else {
                self.to_full_yaml_string(&yaml_path)
            }
        } else {
            self.to_full_yaml_string(&yaml_path)
        };

        fs::write(&yaml_path, final_yaml)
            .map_err(|e| format!("Failed to write config.yaml: {}", e))?;

        // Cleanup any legacy gui_config.json
        let legacy_json = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("monolai")
            .join("gui_config.json");
        if legacy_json.exists() {
            let _ = fs::remove_file(legacy_json);
        }

        Ok(())
    }

    fn to_full_yaml_string(&self, yaml_path: &Path) -> String {
        let host_val = if self.host.trim().is_empty() {
            "0.0.0.0"
        } else {
            self.host.trim()
        };
        render_config_template(
            &yaml_path.to_string_lossy(),
            self.models.trim(),
            self.runtimes.trim(),
            host_val,
            self.port,
            &self.gui,
        )
    }
}

/// Updates or replaces the `gui:` section in a YAML string, preserving other keys and comments.
pub fn update_or_append_gui_section(content: &str, gui: &GuiPreferences) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_gui = false;
    let mut filtered_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed == "# 4. Desktop GUI Preferences:" {
            continue;
        }
        if trimmed.starts_with("minimize_on_start:")
            || trimmed.starts_with("autostart_server:")
            || trimmed.starts_with("autostart_app:")
        {
            continue;
        }
        if trimmed == "gui:" || trimmed.starts_with("gui:") {
            in_gui = true;
            continue;
        }
        if in_gui {
            if line.starts_with("  ") || line.starts_with('\t') || trimmed.is_empty() {
                continue;
            } else {
                in_gui = false;
            }
        }
        filtered_lines.push(line);
    }

    let mut result = filtered_lines.join("\n");
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result.push_str(&format!(
        "\n# 4. Desktop GUI Preferences:\ngui:\n  minimize_on_start: {}\n  autostart_server: {}\n  autostart_app: {}\n",
        gui.minimize_on_start, gui.autostart_server, gui.autostart_app
    ));
    result
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

    // 2. If not found as active key, check if it was commented out: e.g. "# models: /path"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_yaml_field_existing_key() {
        let yaml = "models: /old/path\nport: 8080\n";
        let updated = update_yaml_field(yaml, "models", "/new/path");
        assert!(updated.contains("models: /new/path"));
        assert!(updated.contains("port: 8080"));
    }

    #[test]
    fn test_update_or_append_gui_section() {
        let yaml = "models: /path\nport: 8080\n";
        let gui = GuiPreferences {
            minimize_on_start: false,
            autostart_server: true,
            autostart_app: false,
        };
        let updated = update_or_append_gui_section(yaml, &gui);
        assert!(updated.contains("gui:\n  minimize_on_start: false\n  autostart_server: true\n  autostart_app: false"));
    }

    #[test]
    fn test_gui_config_default_values() {
        let config = GuiConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
        assert!(config.gui.minimize_on_start);
        assert!(config.gui.autostart_server);
        assert!(config.gui.autostart_app);
    }

    #[test]
    fn test_render_template_from_file() {
        let gui = GuiPreferences {
            minimize_on_start: false,
            autostart_server: false,
            autostart_app: false,
        };
        let rendered = render_config_template("/tmp/cfg.yaml", "/m", "/r", "127.0.0.1", 9000, &gui);
        assert!(rendered.contains("models: /m"));
        assert!(rendered.contains("runtimes: /r"));
        assert!(rendered.contains("host: 127.0.0.1"));
        assert!(rendered.contains("port: 9000"));
        assert!(rendered.contains("gui:\n  minimize_on_start: false\n  autostart_server: false\n  autostart_app: false"));
    }
}
