use crate::runtimes::hardware::{detect_hardware, resolve_target_acceleration};
use crate::runtimes::manifest_loader::{get_manifest_for_runtime, RuntimeManifest};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct InstallProgress {
    pub runtime_id: String,
    pub status: String, // "idle", "downloading", "extracting", "completed", "error"
    pub percent: f32,
    pub speed_mbps: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RuntimeItem {
    pub id: String,
    pub name: String,
    pub version: String,
    pub icon: Option<String>,
    pub website: Option<String>,
    pub description: String,
    pub features: Vec<String>,
    pub is_installed: bool,
    pub installed_path: Option<String>,
    pub active_acceleration: String,
    pub install_progress: Option<InstallProgress>,
}

pub struct RuntimeInstallerManager {
    progress_map: Mutex<HashMap<String, InstallProgress>>,
    broadcaster: broadcast::Sender<InstallProgress>,
}

impl RuntimeInstallerManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(128);
        Self {
            progress_map: Mutex::new(HashMap::new()),
            broadcaster: tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<InstallProgress> {
        self.broadcaster.subscribe()
    }

    pub fn get_progress(&self, runtime_id: &str) -> Option<InstallProgress> {
        let map = self.progress_map.lock().unwrap();
        map.get(runtime_id).cloned()
    }

    pub fn update_progress(&self, progress: InstallProgress) {
        {
            let mut map = self.progress_map.lock().unwrap();
            map.insert(progress.runtime_id.clone(), progress.clone());
        }
        let _ = self.broadcaster.send(progress);
    }
}

/// Check if a given runtime is physically installed in the runtimes directory.
pub fn find_installed_binary(runtimes_dir: &Path, manifest: &RuntimeManifest) -> Option<PathBuf> {
    let target_dir = runtimes_dir.join(&manifest.id);
    if !target_dir.exists() {
        return None;
    }

    // 1. Direct path: target_dir/binary_name
    let direct_bin = target_dir.join(&manifest.binary_name);
    if direct_bin.exists() && direct_bin.is_file() {
        return Some(direct_bin);
    }

    // Windows fallback with .exe
    let direct_bin_exe = target_dir.join(format!("{}.exe", &manifest.binary_name));
    if direct_bin_exe.exists() && direct_bin_exe.is_file() {
        return Some(direct_bin_exe);
    }

    // 2. Recursive search within target_dir in case archive had an inner folder
    if let Ok(entries) = walkdir::WalkDir::new(&target_dir).max_depth(3).into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in entries {
            if entry.file_type().is_file() {
                let name = entry.file_name().to_string_lossy();
                if name == manifest.binary_name || name == format!("{}.exe", manifest.binary_name) {
                    return Some(entry.path().to_path_buf());
                }
            }
        }
    }

    None
}

/// Uninstall runtime by removing its folder in runtimes_dir.
pub fn uninstall_runtime(runtimes_dir: &Path, runtime_id: &str) -> Result<(), String> {
    let target_dir = runtimes_dir.join(runtime_id);
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).map_err(|e| format!("Failed to remove runtime directory: {}", e))?;
        tracing::info!("Uninstalled runtime '{}' from {}", runtime_id, target_dir.display());
    }
    Ok(())
}

/// Download and extract a runtime.
pub async fn install_runtime(
    installer_mgr: Arc<RuntimeInstallerManager>,
    runtimes_dir: PathBuf,
    runtime_id: String,
    hardware_override: Option<String>,
) -> Result<PathBuf, String> {
    let manifest = get_manifest_for_runtime(&runtime_id)
        .ok_or_else(|| format!("Runtime manifest for '{}' not found", runtime_id))?;

    let hw_report = detect_hardware();
    let os = hw_report.os.clone();
    let arch = hw_report.arch.clone();
    let acceleration = resolve_target_acceleration(hardware_override.as_deref());

    let download_target = manifest
        .get_download_target(&os, &arch, &acceleration)
        .ok_or_else(|| {
            format!(
                "No download package available for runtime '{}' on {}/{} with {} acceleration",
                runtime_id, os, arch, acceleration
            )
        })?;

    tracing::info!(
        "Starting download for runtime '{}' ({}/{}/{}): {}",
        runtime_id,
        os,
        arch,
        acceleration,
        download_target.url
    );

    // Prepare runtimes directory
    fs::create_dir_all(&runtimes_dir)
        .map_err(|e| format!("Failed to create runtimes directory: {}", e))?;

    let temp_archive_path = runtimes_dir.join(format!(".tmp_download_{}.archive", runtime_id));
    let temp_extract_dir = runtimes_dir.join(format!(".tmp_extract_{}", runtime_id));

    // Cleanup any leftovers from failed previous attempts
    let _ = fs::remove_file(&temp_archive_path);
    let _ = fs::remove_dir_all(&temp_extract_dir);

    // Initial progress
    installer_mgr.update_progress(InstallProgress {
        runtime_id: runtime_id.clone(),
        status: "downloading".to_string(),
        percent: 0.0,
        speed_mbps: 0.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        error_message: None,
    });

    let client = reqwest::Client::builder()
        .user_agent("Monolai-Runtime-Manager/1.0")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(&download_target.url)
        .send()
        .await
        .map_err(|e| format!("Failed to initiate download: {}", e))?;

    if !response.status().is_success() {
        let err = format!(
            "Download failed with HTTP status: {} (URL: {})",
            response.status(),
            download_target.url
        );
        installer_mgr.update_progress(InstallProgress {
            runtime_id: runtime_id.clone(),
            status: "error".to_string(),
            percent: 0.0,
            speed_mbps: 0.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            error_message: Some(err.clone()),
        });
        return Err(err);
    }

    let total_bytes = response.content_length().unwrap_or(0);
    let mut downloaded_bytes: u64 = 0;
    let mut stream = response.bytes_stream();

    let mut file = tokio::fs::File::create(&temp_archive_path)
        .await
        .map_err(|e| format!("Failed to create temporary archive file: {}", e))?;

    let start_time = Instant::now();
    let mut last_progress_report = Instant::now();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Error while streaming download: {}", e))?;
        use tokio::io::AsyncWriteExt;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Error writing chunk to disk: {}", e))?;

        downloaded_bytes += chunk.len() as u64;

        if last_progress_report.elapsed().as_millis() > 200 || downloaded_bytes == total_bytes {
            last_progress_report = Instant::now();
            let elapsed_secs = start_time.elapsed().as_secs_f32();
            let speed_mbps = if elapsed_secs > 0.0 {
                (downloaded_bytes as f32 / (1024.0 * 1024.0)) / elapsed_secs
            } else {
                0.0
            };

            let percent = if total_bytes > 0 {
                (downloaded_bytes as f32 / total_bytes as f32) * 100.0
            } else {
                50.0
            };

            installer_mgr.update_progress(InstallProgress {
                runtime_id: runtime_id.clone(),
                status: "downloading".to_string(),
                percent,
                speed_mbps,
                downloaded_bytes,
                total_bytes,
                error_message: None,
            });
        }
    }

    // Flush file
    use tokio::io::AsyncWriteExt;
    file.flush().await.map_err(|e| format!("Failed to flush archive file: {}", e))?;
    drop(file);

    // Extraction phase
    installer_mgr.update_progress(InstallProgress {
        runtime_id: runtime_id.clone(),
        status: "extracting".to_string(),
        percent: 99.0,
        speed_mbps: 0.0,
        downloaded_bytes,
        total_bytes,
        error_message: None,
    });

    fs::create_dir_all(&temp_extract_dir)
        .map_err(|e| format!("Failed to create temporary extraction directory: {}", e))?;

    let extract_result = tokio::task::spawn_blocking({
        let archive_path = temp_archive_path.clone();
        let extract_dir = temp_extract_dir.clone();
        let archive_type = download_target.archive_type.clone();

        move || -> Result<(), String> {
            if archive_type == "zip" || archive_path.to_string_lossy().ends_with(".zip") {
                let file = File::open(&archive_path).map_err(|e| format!("Failed to open zip archive: {}", e))?;
                let reader = BufReader::new(file);
                let mut zip = zip::ZipArchive::new(reader).map_err(|e| format!("Failed to parse zip archive: {}", e))?;

                for i in 0..zip.len() {
                    let mut zip_file = zip.by_index(i).map_err(|e| format!("Failed to read zip entry: {}", e))?;
                    let outpath = match zip_file.enclosed_name() {
                        Some(path) => extract_dir.join(path),
                        None => continue,
                    };

                    if zip_file.is_dir() {
                        fs::create_dir_all(&outpath).map_err(|e| format!("Failed to create dir: {}", e))?;
                    } else {
                        if let Some(p) = outpath.parent() {
                            if !p.exists() {
                                fs::create_dir_all(p).map_err(|e| format!("Failed to create parent dir: {}", e))?;
                            }
                        }
                        let mut outfile = File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
                        io::copy(&mut zip_file, &mut outfile).map_err(|e| format!("Failed to copy file: {}", e))?;
                    }
                }
            } else if archive_type == "tar.gz" || archive_path.to_string_lossy().ends_with(".tar.gz") {
                let file = File::open(&archive_path).map_err(|e| format!("Failed to open tar.gz archive: {}", e))?;
                let tar = flate2::read::GzDecoder::new(file);
                let mut archive = tar::Archive::new(tar);
                archive.unpack(&extract_dir).map_err(|e| format!("Failed to unpack tar.gz: {}", e))?;
            } else {
                return Err(format!("Unsupported archive format: {}", archive_type));
            }

            Ok(())
        }
    })
    .await
    .map_err(|e| format!("Extraction task panicked: {}", e))?;

    if let Err(e) = extract_result {
        let _ = fs::remove_file(&temp_archive_path);
        let _ = fs::remove_dir_all(&temp_extract_dir);
        installer_mgr.update_progress(InstallProgress {
            runtime_id: runtime_id.clone(),
            status: "error".to_string(),
            percent: 0.0,
            speed_mbps: 0.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            error_message: Some(e.clone()),
        });
        return Err(e);
    }

    // Move to final target directory: runtimes_dir/runtime_id
    let final_dest_dir = runtimes_dir.join(&runtime_id);
    let _ = fs::remove_dir_all(&final_dest_dir);

    // Check if the extracted directory contains a single root folder containing all files
    let mut actual_source_dir = temp_extract_dir.clone();
    if let Ok(entries) = fs::read_dir(&temp_extract_dir) {
        let valid_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        if valid_entries.len() == 1 && valid_entries[0].file_type().map_or(false, |t| t.is_dir()) {
            actual_source_dir = valid_entries[0].path();
        }
    }

    fs::rename(&actual_source_dir, &final_dest_dir)
        .map_err(|e| format!("Failed to move extracted runtime to {}: {}", final_dest_dir.display(), e))?;

    // Cleanup temp files
    let _ = fs::remove_file(&temp_archive_path);
    let _ = fs::remove_dir_all(&temp_extract_dir);

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(entries) = walkdir::WalkDir::new(&final_dest_dir).into_iter().collect::<Result<Vec<_>, _>>() {
            for entry in entries {
                if entry.file_type().is_file() {
                    let file_name = entry.file_name().to_string_lossy();
                    if file_name == manifest.binary_name || file_name.starts_with("llama-") || file_name.starts_with("sd-") {
                        let _ = fs::set_permissions(entry.path(), fs::Permissions::from_mode(0o755));
                    }
                }
            }
        }
    }

    // Verify binary exists
    let resolved_binary = find_installed_binary(&runtimes_dir, &manifest)
        .ok_or_else(|| format!("Binary '{}' not found after extraction in {}", manifest.binary_name, final_dest_dir.display()))?;

    // Final completed progress
    installer_mgr.update_progress(InstallProgress {
        runtime_id: runtime_id.clone(),
        status: "completed".to_string(),
        percent: 100.0,
        speed_mbps: 0.0,
        downloaded_bytes: total_bytes,
        total_bytes,
        error_message: None,
    });

    tracing::info!(
        "Runtime '{}' successfully installed at: {}",
        runtime_id,
        resolved_binary.display()
    );

    Ok(resolved_binary)
}
