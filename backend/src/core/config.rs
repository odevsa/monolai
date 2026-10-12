use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub models: Option<String>,
    pub runtimes: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ConfigStatus {
    pub is_valid: bool,
    pub has_models: bool,
    pub has_runtimes: bool,
    pub created_auto_file: bool,
    pub is_docker: bool,
    pub loaded_path: Option<String>,
    pub expected_path: String,
    pub models_dir: Option<String>,
    pub runtimes_dir: Option<String>,
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
        home.join(".local")
            .join("share")
            .join("monolai")
            .join("runtimes")
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

pub const CONFIG_TEMPLATE: &str = include_str!("../../../config.example.yaml");

pub fn render_config_template(
    config_path: &str,
    models: &str,
    runtimes: &str,
    host: &str,
    port: u16,
    minimize_on_start: bool,
    autostart_server: bool,
    autostart_app: bool,
) -> String {
    CONFIG_TEMPLATE
        .replace("{config_path}", config_path)
        .replace("{models}", models)
        .replace("{runtimes}", runtimes)
        .replace("{host}", host)
        .replace("{port}", &port.to_string())
        .replace("{minimize_on_start}", &minimize_on_start.to_string())
        .replace("{autostart_server}", &autostart_server.to_string())
        .replace("{autostart_app}", &autostart_app.to_string())
}

pub fn generate_commented_example_yaml(
    expected_path: &str,
    models_dir: &str,
    runtimes_dir: &str,
    host: &str,
    port: u16,
) -> String {
    render_config_template(
        expected_path,
        models_dir,
        runtimes_dir,
        host,
        port,
        true,
        true,
        true,
    )
}

pub fn is_running_in_docker() -> bool {
    std::env::var("DOCKER")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
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

    let env_models = std::env::var("MODELS_DIR")
        .ok()
        .filter(|m| !m.trim().is_empty());
    let env_runtimes = std::env::var("RUNTIMES_DIR")
        .ok()
        .filter(|r| !r.trim().is_empty());
    let env_host = std::env::var("HOST").ok().filter(|h| !h.trim().is_empty());
    let env_port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok());

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
        if save_config(
            Some(&expected_path_str),
            &default_models_str,
            &default_runtimes_str,
            Some(&initial_host),
            Some(initial_port),
            None,
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
        let status = ConfigStatus {
            is_valid: false,
            has_models: false,
            has_runtimes: false,
            created_auto_file,
            is_docker: in_docker,
            loaded_path: None,
            expected_path: expected_path_str,
            models_dir: Some(models_dir),
            runtimes_dir: Some(runtimes_dir),
            host: initial_host,
            port: initial_port,
            error_message: Some(
                "Configuration file does not exist yet. Please complete initial setup.".to_string(),
            ),
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
                created_auto_file,
                is_docker: in_docker,
                loaded_path: Some(expected_path_str.clone()),
                expected_path: expected_path_str,
                models_dir: None,
                runtimes_dir: None,
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
                created_auto_file,
                is_docker: in_docker,
                loaded_path: Some(expected_path_str.clone()),
                expected_path: expected_path_str,
                models_dir: None,
                runtimes_dir: None,
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

    let is_valid = has_models && has_runtimes;

    let active_models_dir = config
        .models
        .clone()
        .unwrap_or_else(|| default_models_str.clone());
    let active_runtimes_dir = config
        .runtimes
        .clone()
        .unwrap_or_else(|| default_runtimes_str.clone());
    let active_host = config.host.clone().unwrap_or(initial_host);
    let active_port = config.port.unwrap_or(initial_port);

    let status = ConfigStatus {
        is_valid,
        has_models,
        has_runtimes,
        created_auto_file,
        is_docker: in_docker,
        loaded_path: Some(expected_path_str.clone()),
        expected_path: expected_path_str,
        models_dir: Some(active_models_dir),
        runtimes_dir: Some(active_runtimes_dir),
        host: active_host,
        port: active_port,
        error_message: if is_valid {
            None
        } else {
            Some("Configuration is missing required fields (models or runtimes).".to_string())
        },
        example_yaml,
        cli_command_example,
    };

    (config, status)
}

pub fn update_or_append_gui_section(
    content: &str,
    minimize_on_start: bool,
    autostart_server: bool,
    autostart_app: bool,
) -> String {
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
        minimize_on_start, autostart_server, autostart_app
    ));
    result
}

pub fn update_yaml_field(content: &str, key: &str, value: &str) -> String {
    let mut replaced = false;
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

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

    if !replaced {
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&format!("{}: {}\n", key, value));
    }

    result
}

pub fn save_config(
    explicit_path: Option<&str>,
    models: &str,
    runtimes: &str,
    host: Option<&str>,
    port: Option<u16>,
    gui: Option<(bool, bool, bool)>,
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

    let final_yaml = if target_path.exists() {
        if let Ok(existing) = fs::read_to_string(&target_path) {
            let mut updated = update_yaml_field(&existing, "models", models.trim());
            updated = update_yaml_field(&updated, "runtimes", runtimes.trim());
            updated = update_yaml_field(&updated, "host", h);
            updated = update_yaml_field(&updated, "port", &p.to_string());

            if let Some((min_start, auto_srv, auto_app)) = gui {
                update_or_append_gui_section(&updated, min_start, auto_srv, auto_app)
            } else if !existing.contains("gui:") {
                update_or_append_gui_section(&updated, true, true, true)
            } else {
                updated
            }
        } else {
            let (min_start, auto_srv, auto_app) = gui.unwrap_or((true, true, true));
            render_config_template(
                &target_path.to_string_lossy(),
                models.trim(),
                runtimes.trim(),
                h,
                p,
                min_start,
                auto_srv,
                auto_app,
            )
        }
    } else {
        let (min_start, auto_srv, auto_app) = gui.unwrap_or((true, true, true));
        render_config_template(
            &target_path.to_string_lossy(),
            models.trim(),
            runtimes.trim(),
            h,
            p,
            min_start,
            auto_srv,
            auto_app,
        )
    };

    fs::write(&target_path, final_yaml).map_err(|e| {
        format!(
            "Failed to write configuration file at {}: {}",
            target_path.display(),
            e
        )
    })?;

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
        assert!(!yaml.contains("hardware:"));
        assert!(yaml.contains("port: 8080"));
        assert!(yaml.contains("# 1. Models Directory:"));
        assert!(yaml.contains("# 2. Runtimes Directory:"));
        assert!(yaml.contains("# 3. Network Configuration:"));
        assert!(yaml.contains("# 4. Desktop GUI Preferences:"));
        assert!(yaml.contains("minimize_on_start: true"));
    }

    #[test]
    fn test_save_config_includes_comments_and_gui() {
        let temp_file = std::env::temp_dir().join(format!(
            "monolai_test_config_{}.yaml",
            uuid::Uuid::new_v4()
        ));
        let path_str = temp_file.to_string_lossy().to_string();

        save_config(
            Some(&path_str),
            "/custom/models",
            "/custom/runtimes",
            Some("127.0.0.1"),
            Some(9000),
            Some((false, true, false)),
        )
        .unwrap();

        let content = fs::read_to_string(&temp_file).unwrap();
        assert!(content.contains("# =============================================================================="));
        assert!(content.contains("# 1. Models Directory:"));
        assert!(content.contains("models: /custom/models"));
        assert!(content.contains("# 2. Runtimes Directory:"));
        assert!(content.contains("runtimes: /custom/runtimes"));
        assert!(!content.contains("hardware:"));
        assert!(content.contains("# 3. Network Configuration:"));
        assert!(content.contains("host: 127.0.0.1"));
        assert!(content.contains("port: 9000"));
        assert!(content.contains("# 4. Desktop GUI Preferences:"));
        assert!(content.contains("minimize_on_start: false"));
        assert!(content.contains("autostart_server: true"));
        assert!(content.contains("autostart_app: false"));

        // Now update without GUI parameter, ensure GUI section is preserved!
        save_config(
            Some(&path_str),
            "/custom/models2",
            "/custom/runtimes2",
            Some("0.0.0.0"),
            Some(8080),
            None,
        )
        .unwrap();

        let updated_content = fs::read_to_string(&temp_file).unwrap();
        let _ = fs::remove_file(&temp_file);
        assert!(updated_content.contains("models: /custom/models2"));
        assert!(updated_content.contains("runtimes: /custom/runtimes2"));
        assert!(updated_content.contains("minimize_on_start: false"));
        assert!(updated_content.contains("autostart_server: true"));
        assert!(updated_content.contains("autostart_app: false"));
    }
}
