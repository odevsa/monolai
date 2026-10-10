use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use crate::core::config::GuiConfig;
use crate::platform::process_ext;

pub fn spawn_server_process(
    binary_path: &Path,
    config: &GuiConfig,
) -> Result<(Child, PathBuf), String> {
    let mut cmd = Command::new(binary_path);
    if let Some(parent) = binary_path.parent() {
        cmd.current_dir(parent);
    }
    cmd.env("HOST", &config.host);
    cmd.env("PORT", config.port.to_string());
    cmd.env("HARDWARE", &config.hardware);
    cmd.env("MODELS_DIR", &config.models_dir);
    cmd.env("RUNTIMES_DIR", &config.runtimes_dir);
    cmd.arg("--config").arg(GuiConfig::backend_yaml_path());

    // Redirect child stdout and stderr to server.log to avoid invalid handle crashes on Windows GUI
    let log_dir = dirs::data_local_dir()
        .map(|p| p.join("monolai").join("logs"))
        .unwrap_or_else(|| PathBuf::from("logs"));
    let _ = std::fs::create_dir_all(&log_dir);
    let log_file_path = log_dir.join("server.log");

    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_file_path)
    {
        if let Ok(file_clone) = file.try_clone() {
            cmd.stdout(file);
            cmd.stderr(file_clone);
        } else {
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());
        }
    } else {
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
    }

    process_ext::configure_background_command(&mut cmd);

    let spawned = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", binary_path.display(), e))?;

    process_ext::assign_to_job_object(&spawned);

    Ok((spawned, log_file_path))
}

pub fn stop_server_process(child: &mut Child) -> Result<(), String> {
    #[cfg(unix)]
    {
        let pid = child.id() as i32;
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
    }

    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }

    // Give child up to 2.5 seconds to gracefully exit
    for _ in 0..25 {
        if let Ok(Some(_)) = child.try_wait() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Force kill if still lingering
    if let Ok(None) = child.try_wait() {
        let _ = child.kill();
        let _ = child.wait();
    }

    Ok(())
}

pub fn run_maintenance_command(
    binary_path: &Path,
    flag: &str,
    config: Option<&GuiConfig>,
) -> Result<String, String> {
    let mut cmd = Command::new(binary_path);
    if let Some(parent) = binary_path.parent() {
        cmd.current_dir(parent);
    }
    cmd.arg(flag);

    if let Some(cfg) = config {
        cmd.arg("--config").arg(GuiConfig::backend_yaml_path());
        cmd.env("HOST", &cfg.host);
        cmd.env("PORT", cfg.port.to_string());
        cmd.env("HARDWARE", &cfg.hardware);
        cmd.env("MODELS_DIR", &cfg.models_dir);
        cmd.env("RUNTIMES_DIR", &cfg.runtimes_dir);
    }

    process_ext::configure_background_command(&mut cmd);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", binary_path.display(), e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        let err_text = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("Process exited with code {:?}", output.status.code())
        };
        return Err(err_text);
    }

    let last_line = stdout
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("Operation completed successfully.")
        .to_string();

    Ok(last_line)
}

pub fn read_recent_log_errors(log_path: &Path) -> Option<String> {
    std::fs::read_to_string(log_path).ok().and_then(|content| {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(
                trimmed
                    .lines()
                    .rev()
                    .take(3)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        }
    })
}
