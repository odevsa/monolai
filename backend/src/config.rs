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

pub fn is_running_in_docker() -> bool {
    std::env::var("DOCKER").map(|v| v == "true" || v == "1").unwrap_or(false)
        || Path::new("/.dockerenv").exists()
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

    let example_yaml = generate_commented_example_yaml(&expected_path_str, &default_models_str, &default_runtimes_str);
    let cli_command_example = format!("monolai --config {}", expected_path_str);

    let env_models = std::env::var("MODELS_DIR").ok().filter(|m| !m.trim().is_empty());
    let env_runtimes = std::env::var("RUNTIMES_DIR").ok().filter(|r| !r.trim().is_empty());
    let env_hardware = std::env::var("HARDWARE").ok().filter(|h| !h.trim().is_empty());
    let in_docker = is_running_in_docker();
    let mut created_auto_file = false;

    // In Docker, automatically create the configuration with container volumes
    if !expected_path_buf.exists() && in_docker {
        let hw_to_save = env_hardware.clone().unwrap_or_default();
        if let Ok(_) = save_config(
            Some(&expected_path_str),
            &default_models_str,
            &default_runtimes_str,
            &hw_to_save,
        ) {
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
            loaded_path: None,
            expected_path: expected_path_str,
            models_dir: Some(models_dir),
            runtimes_dir: Some(runtimes_dir),
            hardware,
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
                hardware: env_hardware.unwrap_or_else(|| "auto".to_string()),
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
                loaded_path: Some(expected_path_str.clone()),
                expected_path: expected_path_str,
                models_dir: None,
                runtimes_dir: None,
                hardware: env_hardware.unwrap_or_else(|| "auto".to_string()),
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

    let resolved_models_dir = config.models.as_ref().map(|m| expand_tilde(m).to_string_lossy().to_string());
    let resolved_runtimes_dir = config.runtimes.as_ref().map(|r| expand_tilde(r).to_string_lossy().to_string());
    let resolved_hardware = config.hardware.clone().unwrap_or_else(|| "auto".to_string());

    let error_message = if is_valid {
        None
    } else if !has_models && !has_runtimes {
        Some("Both 'models' directory and 'runtimes' directory must be specified.".to_string())
    } else if !has_models {
        Some("Missing 'models' directory specification in configuration.".to_string())
    } else if !has_runtimes {
        Some("Missing 'runtimes' directory specification in configuration.".to_string())
    } else if !has_hardware {
        Some("Hardware acceleration target must be configured.".to_string())
    } else {
        None
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

/// Save new configuration to file while preserving existing comments.
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

    let final_yaml = if target_path.exists() {
        if let Ok(existing) = fs::read_to_string(&target_path) {
            let mut updated = update_yaml_field(&existing, "models", models.trim());
            updated = update_yaml_field(&updated, "runtimes", runtimes.trim());
            let hw_val = if hardware.trim().is_empty() { "auto" } else { hardware.trim() };
            update_yaml_field(&updated, "hardware", hw_val)
        } else {
            generate_commented_example_yaml(&target_path.to_string_lossy(), models, runtimes)
        }
    } else {
        let yaml_content = generate_commented_example_yaml(
            &target_path.to_string_lossy(),
            models,
            runtimes,
        );
        if hardware.trim().is_empty() {
            yaml_content.replace("hardware: auto\n", "# hardware: auto\n")
        } else if hardware != "auto" {
            yaml_content.replace("hardware: auto", &format!("hardware: {}", hardware))
        } else {
            yaml_content
        }
    };

    fs::write(&target_path, final_yaml).map_err(|e| format!("Failed to write configuration file: {}", e))?;

    tracing::info!("Saved configuration file to {}", target_path.display());
    Ok(target_path)
}
