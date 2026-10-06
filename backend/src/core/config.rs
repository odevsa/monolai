use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub models: Option<String>,
    pub runtimes: Option<String>,
    pub hardware: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ConfigStatus {
    pub is_valid: bool,
    pub has_models: bool,
    pub has_runtimes: bool,
    pub has_hardware: bool,
    pub created_auto_file: bool,
    pub is_docker: bool,
    pub loaded_path: Option<String>,
    pub expected_path: String,
    pub models_dir: Option<String>,
    pub runtimes_dir: Option<String>,
    pub hardware: String,
    pub host: String,
    pub port: u16,
    pub error_message: Option<String>,
    pub example_yaml: String,
    pub cli_command_example: String,
}

/// Centralized helper for resolving system and user directories with tilde expansion.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PathResolver {
    pub models_dir: PathBuf,
    pub runtimes_dir: PathBuf,
    pub db_path: PathBuf,
    pub config_path: PathBuf,
}

impl PathResolver {
    pub fn new(
        config: &AppConfig,
        cli_config: Option<&str>,
        cli_models: Option<&str>,
        cli_runtimes: Option<&str>,
        cli_db: Option<&str>,
    ) -> Self {
        let config_path = if let Some(p) = cli_config {
            expand_tilde(p)
        } else {
            get_default_config_path()
        };

        let models_dir = if let Some(m) = cli_models {
            expand_tilde(m)
        } else if let Some(ref m) = config.models {
            expand_tilde(m)
        } else {
            get_default_models_dir()
        };

        let runtimes_dir = if let Some(r) = cli_runtimes {
            expand_tilde(r)
        } else if let Some(ref r) = config.runtimes {
            expand_tilde(r)
        } else {
            get_default_runtimes_dir()
        };

        let db_path = if let Some(d) = cli_db {
            expand_tilde(d)
        } else {
            get_default_db_path()
        };

        Self {
            models_dir,
            runtimes_dir,
            db_path,
            config_path,
        }
    }
}

pub fn get_env_data_dir() -> Option<PathBuf> {
    std::env::var("MONOLAI_DATA_DIR")
        .or_else(|_| std::env::var("DATA_DIR"))
        .ok()
        .map(PathBuf::from)
}

pub fn get_default_config_path() -> PathBuf {
    if let Some(data_dir) = get_env_data_dir() {
        data_dir.join("config.yaml")
    } else if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("monolai").join("config.yaml")
    } else {
        PathBuf::from("config.yaml")
    }
}

pub fn get_default_models_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MODELS_DIR") {
        PathBuf::from(dir)
    } else if let Some(data_dir) = get_env_data_dir() {
        data_dir.join("models")
    } else if let Some(data_dir) = dirs::data_local_dir() {
        data_dir.join("monolai").join("models")
    } else if let Some(home) = dirs::home_dir() {
        home.join("models")
    } else {
        PathBuf::from("models")
    }
}

pub fn get_default_runtimes_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("RUNTIMES_DIR") {
        PathBuf::from(dir)
    } else if let Some(data_dir) = get_env_data_dir() {
        data_dir.join("runtimes")
    } else if let Some(data_dir) = dirs::data_local_dir() {
        data_dir.join("monolai").join("runtimes")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".local").join("share").join("monolai").join("runtimes")
    } else {
        PathBuf::from("runtimes")
    }
}

pub fn get_default_db_path() -> PathBuf {
    if let Some(data_dir) = get_env_data_dir() {
        data_dir.join("app.db")
    } else if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("monolai").join("app.db")
    } else {
        PathBuf::from("app.db")
    }
}

pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if let Ok(stripped) = p.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    p.to_path_buf()
}

pub fn generate_commented_example_yaml(
    expected_path: &str,
    models_dir: &str,
    runtimes_dir: &str,
    host: &str,
    port: u16,
) -> String {
    format!(
        r#"# ==============================================================================
# Monolai Configuration File
# ==============================================================================
# Target file path: {}
#
# 1. Models Directory:
# Directory for storing local AI model files (.gguf, .safetensors).
models: {}

# 2. Runtimes Directory:
# Directory where upstream runtime engines (llama-cpp, sd-cpp) are installed and managed.
runtimes: {}

# 3. Hardware Acceleration:
# Preferred target acceleration: auto, cpu, cuda, rocm, vulkan, oneapi
hardware: auto

# 4. Network Configuration:
# Host address and HTTP port for the Monolai server
host: {}
port: {}
"#,
        expected_path, models_dir, runtimes_dir, host, port
    )
}

pub fn is_running_in_docker() -> bool {
    std::env::var("DOCKER").map(|v| v == "true" || v == "1").unwrap_or(false)
        || Path::new("/.dockerenv").exists()
        || Path::new("/run/.containerenv").exists()
        || (std::env::var("MODELS_DIR").as_deref() == Ok("/app/models")
            && std::env::var("RUNTIMES_DIR").as_deref() == Ok("/app/runtimes"))
}

pub fn load_config(explicit_path: Option<&str>) -> (AppConfig, ConfigStatus) {
    let expected_path_buf = if let Some(p) = explicit_path {
        expand_tilde(p)
    } else {
        get_default_config_path()
    };

    let expected_path_str = expected_path_buf.to_string_lossy().to_string();
    let default_models_str = get_default_models_dir().to_string_lossy().to_string();
    let default_runtimes_str = get_default_runtimes_dir().to_string_lossy().to_string();
    let default_host = "0.0.0.0".to_string();
    let default_port = 8080u16;

    let env_models = std::env::var("MODELS_DIR").ok().filter(|m| !m.trim().is_empty());
    let env_runtimes = std::env::var("RUNTIMES_DIR").ok().filter(|r| !r.trim().is_empty());
    let env_hardware = std::env::var("HARDWARE").ok().filter(|h| !h.trim().is_empty());
    let env_host = std::env::var("HOST").ok().filter(|h| !h.trim().is_empty());
    let env_port = std::env::var("PORT").ok().and_then(|p| p.parse::<u16>().ok());

    let initial_host = env_host.clone().unwrap_or_else(|| default_host.clone());
    let initial_port = env_port.unwrap_or(default_port);

    let example_yaml = generate_commented_example_yaml(
        &expected_path_str,
        &default_models_str,
        &default_runtimes_str,
        &initial_host,
        initial_port,
    );
    let cli_command_example = format!("monolai --config {}", expected_path_str);

    let in_docker = is_running_in_docker();
    let mut created_auto_file = false;

    // In Docker, automatically create the configuration with container volumes
    if !expected_path_buf.exists() && in_docker {
        let hw_to_save = env_hardware.clone().unwrap_or_default();
        if save_config(
            Some(&expected_path_str),
            &default_models_str,
            &default_runtimes_str,
            &hw_to_save,
            Some(&initial_host),
            Some(initial_port),
        )
        .is_ok()
        {
            created_auto_file = true;
            tracing::info!(
                "Docker environment detected: initialized configuration file at {}",
                expected_path_str
            );
        }
    }

    if !expected_path_buf.exists() {
        let models_dir = env_models.unwrap_or(default_models_str);
        let runtimes_dir = env_runtimes.unwrap_or(default_runtimes_str);
        let hardware = env_hardware.unwrap_or_else(|| "auto".to_string());
        let status = ConfigStatus {
            is_valid: false,
            has_models: false,
            has_runtimes: false,
            has_hardware: false,
            created_auto_file,
            is_docker: in_docker,
            loaded_path: None,
            expected_path: expected_path_str,
            models_dir: Some(models_dir),
            runtimes_dir: Some(runtimes_dir),
            hardware,
            host: initial_host,
            port: initial_port,
            error_message: Some("Configuration file does not exist yet. Please complete initial setup.".to_string()),
            example_yaml,
            cli_command_example,
        };
        return (AppConfig::default(), status);
    }

    let contents = match fs::read_to_string(&expected_path_buf) {
        Ok(c) => c,
        Err(e) => {
            let status = ConfigStatus {
                is_valid: false,
                has_models: false,
                has_runtimes: false,
                has_hardware: false,
                created_auto_file,
                is_docker: in_docker,
                loaded_path: Some(expected_path_str.clone()),
                expected_path: expected_path_str,
                models_dir: None,
                runtimes_dir: None,
                hardware: env_hardware.unwrap_or_else(|| "auto".to_string()),
                host: initial_host,
                port: initial_port,
                error_message: Some(format!("Failed to read config file: {}", e)),
                example_yaml,
                cli_command_example,
            };
            return (AppConfig::default(), status);
        }
    };

    let mut config: AppConfig = match serde_yaml::from_str(&contents) {
        Ok(cfg) => cfg,
        Err(e) => {
            let status = ConfigStatus {
                is_valid: false,
                has_models: false,
                has_runtimes: false,
                has_hardware: false,
                created_auto_file,
                is_docker: in_docker,
                loaded_path: Some(expected_path_str.clone()),
                expected_path: expected_path_str,
                models_dir: None,
                runtimes_dir: None,
                hardware: env_hardware.unwrap_or_else(|| "auto".to_string()),
                host: initial_host,
                port: initial_port,
                error_message: Some(format!("Invalid YAML configuration structure: {}", e)),
                example_yaml,
                cli_command_example,
            };
            return (AppConfig::default(), status);
        }
    };

    if let Some(dir) = env_models {
        config.models = Some(dir);
    }
    if let Some(dir) = env_runtimes {
        config.runtimes = Some(dir);
    }
    if let Some(hw) = env_hardware {
        config.hardware = Some(hw);
    }
    if let Some(h) = env_host {
        config.host = Some(h);
    }
    if let Some(p) = env_port {
        config.port = Some(p);
    }

    let has_models = config
        .models
        .as_ref()
        .map_or(false, |m| !m.trim().is_empty());

    let has_runtimes = config
        .runtimes
        .as_ref()
        .map_or(false, |r| !r.trim().is_empty());

    let has_hardware = config
        .hardware
        .as_ref()
        .map_or(false, |h| !h.trim().is_empty());

    let is_valid = has_models && has_runtimes && has_hardware;

    let active_models_dir = config
        .models
        .clone()
        .unwrap_or_else(|| default_models_str.clone());
    let active_runtimes_dir = config
        .runtimes
        .clone()
        .unwrap_or_else(|| default_runtimes_str.clone());
    let active_hardware = config
        .hardware
        .clone()
        .unwrap_or_else(|| "auto".to_string());
    let active_host = config.host.clone().unwrap_or(initial_host);
    let active_port = config.port.unwrap_or(initial_port);

    let status = ConfigStatus {
        is_valid,
        has_models,
        has_runtimes,
        has_hardware,
        created_auto_file,
        is_docker: in_docker,
        loaded_path: Some(expected_path_str.clone()),
        expected_path: expected_path_str,
        models_dir: Some(active_models_dir),
        runtimes_dir: Some(active_runtimes_dir),
        hardware: active_hardware,
        host: active_host,
        port: active_port,
        error_message: if is_valid {
            None
        } else {
            Some("Configuration is missing required fields (models, runtimes, or hardware).".to_string())
        },
        example_yaml,
        cli_command_example,
    };

    (config, status)
}

pub fn save_config(
    explicit_path: Option<&str>,
    models: &str,
    runtimes: &str,
    hardware: &str,
    host: Option<&str>,
    port: Option<u16>,
) -> Result<(), String> {
    let target_path = if let Some(p) = explicit_path {
        expand_tilde(p)
    } else {
        get_default_config_path()
    };

    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory structure for config: {}", e))?;
        }
    }

    let default_host = "0.0.0.0";
    let default_port = 8080u16;

    let h = host.unwrap_or(default_host);
    let p = port.unwrap_or(default_port);
    let hw = if hardware.trim().is_empty() {
        "auto"
    } else {
        hardware.trim()
    };

    let yaml_content = format!(
        r#"# ==============================================================================
# Monolai Configuration File
# Auto-generated by Setup
# ==============================================================================
models: {}
runtimes: {}
hardware: {}
host: {}
port: {}
"#,
        models.trim(),
        runtimes.trim(),
        hw,
        h,
        p
    );

    fs::write(&target_path, yaml_content)
        .map_err(|e| format!("Failed to write configuration file at {}: {}", target_path.display(), e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let p = expand_tilde("models/llama");
        assert_eq!(p, PathBuf::from("models/llama"));
    }

    #[test]
    fn test_generate_commented_example_yaml() {
        let yaml = generate_commented_example_yaml(
            "/tmp/config.yaml",
            "/tmp/models",
            "/tmp/runtimes",
            "127.0.0.1",
            8080,
        );
        assert!(yaml.contains("models: /tmp/models"));
        assert!(yaml.contains("runtimes: /tmp/runtimes"));
        assert!(yaml.contains("port: 8080"));
    }
}
