use std::env;
use std::path::{Path, PathBuf};

/// Locates the `monolai` backend server executable.
pub fn find_server_binary() -> Result<PathBuf, String> {
    let current_exe = env::current_exe().map_err(|e| e.to_string())?;
    let current_dir = current_exe.parent().unwrap_or_else(|| Path::new("."));

    // 1. Check in the same directory as this GUI binary (installed/packaged mode)
    let local_bin = if cfg!(windows) {
        current_dir.join("monolai.exe")
    } else {
        current_dir.join("monolai")
    };
    if local_bin.exists() {
        return Ok(local_bin);
    }

    // 2. Check development target directories
    let dev_paths = [
        current_dir.join("../../../backend/target/release/monolai"),
        current_dir.join("../../../backend/target/debug/monolai"),
        current_dir.join("../target/release/monolai"),
        current_dir.join("../target/debug/monolai"),
        PathBuf::from("target/release/monolai"),
        PathBuf::from("target/debug/monolai"),
        PathBuf::from("backend/target/release/monolai"),
        PathBuf::from("backend/target/debug/monolai"),
    ];

    for p in &dev_paths {
        if p.exists() {
            return Ok(p.clone());
        }
    }

    // 3. Check system PATH
    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            let p = if cfg!(windows) {
                dir.join("monolai.exe")
            } else {
                dir.join("monolai")
            };
            if p.exists() {
                return Ok(p);
            }
        }
    }

    Err("Could not find 'monolai' server binary in current dir, PATH, or target/".to_string())
}
