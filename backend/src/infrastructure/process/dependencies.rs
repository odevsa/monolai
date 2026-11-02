use crate::domain::runtimes::SystemDependency;
use std::collections::HashMap;
use std::path::Path;

/// Detects a canonical platform/distribution key for package manager instructions.
/// Returns keys such as "arch", "ubuntu", "debian", "fedora", "macos", "windows", or "linux".
pub fn detect_current_platform_key() -> String {
    let os = std::env::consts::OS;
    if os == "linux" {
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let mut id = String::new();
            let mut id_like = String::new();
            for line in content.lines() {
                if let Some(val) = line.strip_prefix("ID=") {
                    id = val.trim_matches('"').trim_matches('\'').to_lowercase();
                } else if let Some(val) = line.strip_prefix("ID_LIKE=") {
                    id_like = val.trim_matches('"').trim_matches('\'').to_lowercase();
                }
            }
            if id == "arch" || id_like.contains("arch") || id == "manjaro" || id == "endeavouros" {
                return "arch".to_string();
            }
            if id == "ubuntu" || id == "pop" || id == "linuxmint" {
                return "ubuntu".to_string();
            }
            if id == "debian" || id_like.contains("debian") {
                return "debian".to_string();
            }
            if id == "fedora" || id_like.contains("fedora") || id == "rhel" || id == "centos" {
                return "fedora".to_string();
            }
            if !id.is_empty() {
                return id;
            }
        }
        "linux".to_string()
    } else if os == "macos" {
        "macos".to_string()
    } else if os == "windows" {
        "windows".to_string()
    } else {
        os.to_string()
    }
}

/// Resolves the most specific installation command from instructions map based on detected platform.
pub fn resolve_install_command(
    instructions: &HashMap<String, String>,
    platform_key: &str,
) -> Option<String> {
    if let Some(cmd) = instructions.get(platform_key) {
        return Some(cmd.clone());
    }
    // Context-sensitive fallbacks
    if platform_key == "arch" {
        return instructions.get("arch").cloned();
    }
    if platform_key == "ubuntu" {
        return instructions
            .get("ubuntu")
            .or_else(|| instructions.get("debian"))
            .cloned();
    }
    if platform_key == "debian" {
        return instructions
            .get("debian")
            .or_else(|| instructions.get("ubuntu"))
            .cloned();
    }
    instructions
        .get("linux")
        .or_else(|| instructions.values().next())
        .cloned()
}

/// Verifies whether a declared system dependency is already installed on the host or inside runtime dir.
pub fn check_system_dependency_satisfied(
    dep: &SystemDependency,
    runtime_dir: Option<&Path>,
) -> bool {
    // 1. Check if a library is specified and available
    if let Some(ref lib_name) = dep.check_library {
        // Check local runtime folder first (portable installation)
        if let Some(r_dir) = runtime_dir {
            let lib_dir = r_dir.join("lib");
            let candidates = [
                lib_dir.join(format!("{}.so", lib_name)),
                lib_dir.join(format!("{}.so.1", lib_name)),
                lib_dir.join(format!("{}.dylib", lib_name)),
                lib_dir.join(format!("{}.dll", lib_name)),
                r_dir.join(format!("{}.so", lib_name)),
                r_dir.join(format!("{}.so.1", lib_name)),
                r_dir.join(format!("{}.dylib", lib_name)),
                r_dir.join(format!("{}.dll", lib_name)),
            ];
            if candidates.iter().any(|c| c.exists()) {
                return true;
            }
        }

        // Check system libraries
        #[cfg(target_os = "linux")]
        {
            let system_paths = [
                format!("/usr/lib/{}.so", lib_name),
                format!("/usr/lib/{}.so.1", lib_name),
                format!("/usr/lib64/{}.so", lib_name),
                format!("/usr/lib64/{}.so.1", lib_name),
                format!("/usr/local/lib/{}.so", lib_name),
                format!("/usr/local/lib/{}.so.1", lib_name),
                format!("/usr/lib/x86_64-linux-gnu/{}.so", lib_name),
                format!("/usr/lib/x86_64-linux-gnu/{}.so.1", lib_name),
                format!("/usr/lib/aarch64-linux-gnu/{}.so", lib_name),
                format!("/usr/lib/aarch64-linux-gnu/{}.so.1", lib_name),
            ];
            if system_paths.iter().any(|p| std::path::Path::new(p).exists()) {
                return true;
            }
            if let Ok(output) = std::process::Command::new("ldconfig").arg("-p").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains(lib_name) {
                    return true;
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let macos_paths = [
                format!("/opt/homebrew/lib/{}.dylib", lib_name),
                format!("/usr/local/lib/{}.dylib", lib_name),
                format!("/usr/lib/{}.dylib", lib_name),
            ];
            if macos_paths.iter().any(|p| std::path::Path::new(p).exists()) {
                return true;
            }
        }

        #[cfg(target_os = "windows")]
        {
            if is_binary_in_path(&format!("{}.dll", lib_name)) {
                return true;
            }
        }
    }

    // 2. Check if a binary command is specified and available on PATH
    if let Some(ref bin_name) = dep.check_binary {
        if is_binary_in_path(bin_name) {
            return true;
        }
    }

    false
}

fn is_binary_in_path(bin_name: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(bin_name);
            let candidate_exe = dir.join(format!("{}.exe", bin_name));
            if candidate.is_file() || candidate_exe.is_file() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_install_command_exact_and_fallback() {
        let mut map = HashMap::new();
        map.insert("arch".to_string(), "sudo pacman -S espeak-ng".to_string());
        map.insert(
            "ubuntu".to_string(),
            "sudo apt install espeak-ng".to_string(),
        );

        assert_eq!(
            resolve_install_command(&map, "arch"),
            Some("sudo pacman -S espeak-ng".to_string())
        );
        assert_eq!(
            resolve_install_command(&map, "ubuntu"),
            Some("sudo apt install espeak-ng".to_string())
        );
        assert_eq!(
            resolve_install_command(&map, "debian"),
            Some("sudo apt install espeak-ng".to_string())
        );
    }
}
