use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub models: Option<String>,
    pub runtimes: Option<String>,
    pub hardware: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ConfigStatus {
    pub is_valid: bool,
    pub has_models: bool,
    pub has_runtimes: bool,
    pub has_hardware: bool,
    pub created_auto_file: bool,
    pub loaded_path: Option<String>,
    pub expected_path: String,
    pub models_dir: Option<String>,
    pub runtimes_dir: Option<String>,
    pub hardware: String,
    pub error_message: Option<String>,
    pub example_yaml: String,
    pub cli_command_example: String,
}

pub fn get_default_config_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("monolai").join("config.yaml")
    } else {
        PathBuf::from("config.yaml")
    }
}

pub fn get_default_models_dir() -> PathBuf {
    if let Some(data_dir) = dirs::data_local_dir() {
        data_dir.join("monolai").join("models")
    } else if let Some(home) = dirs::home_dir() {
        home.join("models")
    } else {
        PathBuf::from("models")
    }
}

pub fn get_default_runtimes_dir() -> PathBuf {
    if let Some(data_dir) = dirs::data_local_dir() {
        data_dir.join("monolai").join("runtimes")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".local").join("share").join("monolai").join("runtimes")
    } else {
        PathBuf::from("runtimes")
    }
}

pub fn get_default_db_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
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

pub fn generate_commented_example_yaml(expected_path: &str, models_dir: &str, runtimes_dir: &str) -> String {
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
"#,
        expected_path, models_dir, runtimes_dir
    )
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

    let example_yaml = generate_commented_example_yaml(&expected_path_str, &default_models_str, &default_runtimes_str);
    let cli_command_example = format!("monolai --config {}", expected_path_str);

    let created_auto_file = false;

    if !expected_path_buf.exists() {
        let status = ConfigStatus {
            is_valid: false,
            has_models: false,
            has_runtimes: false,
            has_hardware: false,
            created_auto_file,
            loaded_path: None,
            expected_path: expected_path_str,
            models_dir: Some(default_models_str),
            runtimes_dir: Some(default_runtimes_str),
            hardware: "auto".to_string(),
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
                loaded_path: Some(expected_path_str.clone()),
                expected_path: expected_path_str,
                models_dir: None,
                runtimes_dir: None,
                hardware: "auto".to_string(),
                error_message: Some(format!("Failed to read config file: {}", e)),
                example_yaml,
                cli_command_example,
            };
            return (AppConfig::default(), status);
        }
    };

    let config: AppConfig = match serde_yaml::from_str(&contents) {
        Ok(cfg) => cfg,
        Err(e) => {
            let status = ConfigStatus {
                is_valid: false,
                has_models: false,
                has_runtimes: false,
                has_hardware: false,
                created_auto_file,
                loaded_path: Some(expected_path_str.clone()),
                expected_path: expected_path_str,
                models_dir: None,
                runtimes_dir: None,
                hardware: "auto".to_string(),
                error_message: Some(format!("Invalid YAML configuration structure: {}", e)),
                example_yaml,
                cli_command_example,
            };
            return (AppConfig::default(), status);
        }
    };

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

    let is_valid = has_models && has_runtimes;

    let resolved_models_dir = config.models.as_ref().map(|m| expand_tilde(m).to_string_lossy().to_string());
    let resolved_runtimes_dir = config.runtimes.as_ref().map(|r| expand_tilde(r).to_string_lossy().to_string());
    let resolved_hardware = config.hardware.clone().unwrap_or_else(|| "auto".to_string());

    let error_message = if is_valid {
        None
    } else if !has_models && !has_runtimes {
        Some("Both 'models' directory and 'runtimes' directory must be specified.".to_string())
    } else if !has_models {
        Some("Missing 'models' directory specification in configuration.".to_string())
    } else {
        Some("Missing 'runtimes' directory specification in configuration.".to_string())
    };

    let status = ConfigStatus {
        is_valid,
        has_models,
        has_runtimes,
        has_hardware,
        created_auto_file,
        loaded_path: Some(expected_path_str.clone()),
        expected_path: expected_path_str,
        models_dir: resolved_models_dir,
        runtimes_dir: resolved_runtimes_dir,
        hardware: resolved_hardware,
        error_message,
        example_yaml,
        cli_command_example,
    };

    (config, status)
}

/// Save new configuration to file.
pub fn save_config(
    explicit_path: Option<&str>,
    models: &str,
    runtimes: &str,
    hardware: &str,
) -> Result<PathBuf, String> {
    let target_path = if let Some(p) = explicit_path {
        expand_tilde(p)
    } else {
        get_default_config_path()
    };

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let models_expanded = expand_tilde(models);
    fs::create_dir_all(&models_expanded).map_err(|e| format!("Failed to create models directory: {}", e))?;

    let runtimes_expanded = expand_tilde(runtimes);
    fs::create_dir_all(&runtimes_expanded).map_err(|e| format!("Failed to create runtimes directory: {}", e))?;

    let yaml_content = generate_commented_example_yaml(
        &target_path.to_string_lossy(),
        models,
        runtimes,
    );

    // If hardware is specified, update or append it
    let final_yaml = if hardware != "auto" {
        yaml_content.replace("hardware: auto", &format!("hardware: {}", hardware))
    } else {
        yaml_content
    };

    fs::write(&target_path, final_yaml).map_err(|e| format!("Failed to write configuration file: {}", e))?;

    tracing::info!("Saved configuration file to {}", target_path.display());
    Ok(target_path)
}
