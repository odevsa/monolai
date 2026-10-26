use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

impl GuiConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("monolai")
            .join("config.yaml")
    }

    /// Loads configuration via CLI (`monolai config show --json`).
    /// Falls back to default values if the CLI command fails.
    pub fn load() -> Self {
        Self::load_via_cli().unwrap_or_else(|e| {
            eprintln!(
                "Warning: failed to load config via CLI: {}. Using default.",
                e
            );
            Self::default()
        })
    }

    /// Explicitly loads configuration from backend CLI.
    #[allow(dead_code)]
    pub fn load_via_cli() -> Result<Self, String> {
        let binary = crate::supervisor::binary_locator::find_server_binary()?;
        let mut cmd = std::process::Command::new(binary);
        cmd.args(["config", "show", "--json"]);
        crate::platform::process_ext::configure_background_command(&mut cmd);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute monolai config show: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("monolai config show failed: {}", err));
        }

        let cfg = serde_json::from_slice::<Self>(&output.stdout)
            .map_err(|e| format!("Failed to parse config JSON from CLI: {}", e))?;
        Ok(cfg)
    }

    /// Saves configuration via CLI (`monolai config set ...`).
    pub fn save(&self) -> Result<(), String> {
        self.save_via_cli()
    }

    /// Explicitly saves configuration to backend CLI.
    #[allow(dead_code)]
    pub fn save_via_cli(&self) -> Result<(), String> {
        let binary = crate::supervisor::binary_locator::find_server_binary()?;
        let mut cmd = std::process::Command::new(binary);
        cmd.args(["config", "set"])
            .arg("--models")
            .arg(&self.models)
            .arg("--runtimes")
            .arg(&self.runtimes)
            .arg("--host")
            .arg(&self.host)
            .arg("--port")
            .arg(self.port.to_string())
            .arg("--minimize-on-start")
            .arg(self.gui.minimize_on_start.to_string())
            .arg("--autostart-server")
            .arg(self.gui.autostart_server.to_string())
            .arg("--autostart-app")
            .arg(self.gui.autostart_app.to_string());
        crate::platform::process_ext::configure_background_command(&mut cmd);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute monolai config set: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("monolai config set failed: {}", err));
        }
        Ok(())
    }

    /// Initializes configuration via CLI (`monolai config init`).
    #[allow(dead_code)]
    pub fn init_via_cli() -> Result<(), String> {
        let binary = crate::supervisor::binary_locator::find_server_binary()?;
        let mut cmd = std::process::Command::new(binary);
        cmd.args(["config", "init"]);
        crate::platform::process_ext::configure_background_command(&mut cmd);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute monolai config init: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("monolai config init failed: {}", err));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_gui_config_json_roundtrip() {
        let config = GuiConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GuiConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.port, config.port);
        assert_eq!(deserialized.host, config.host);
        assert_eq!(deserialized.gui, config.gui);
    }
}
