use crate::domain::{InstallProgress, RuntimeManifest};
use crate::infrastructure::hardware::{detect_hardware, resolve_target_acceleration};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::broadcast;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMetadata {
    pub runtime_id: String,
    pub version: String,
    pub acceleration: String,
    pub installed_at: u64,
}

pub fn get_installed_metadata(runtimes_dir: &Path, runtime_id: &str) -> Option<InstalledMetadata> {
    let meta_path = runtimes_dir.join(runtime_id).join(".metadata.json");
    if let Ok(content) = fs::read_to_string(meta_path) {
        if let Ok(meta) = serde_json::from_str::<InstalledMetadata>(&content) {
            return Some(meta);
        }
    }
    None
}

fn save_installed_metadata(dest_dir: &Path, runtime_id: &str, version: &str, acceleration: &str) {
    let meta_path = dest_dir.join(".metadata.json");
    let json = serde_json::json!({
        "runtime_id": runtime_id,
        "version": version,
        "acceleration": acceleration,
        "installed_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    });
    let _ = fs::write(meta_path, json.to_string());
}

fn flatten_single_child_dir(dir: &Path) -> io::Result<()> {
    let entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect();

    if entries.len() == 1 && entries[0].file_type().map_or(false, |t| t.is_dir()) {
        let child_dir = entries[0].path();
        let sub_entries: Vec<_> = fs::read_dir(&child_dir)?
            .filter_map(|e| e.ok())
            .collect();
        for sub in sub_entries {
            let dest = dir.join(sub.file_name());
            fs::rename(sub.path(), dest)?;
        }
        let _ = fs::remove_dir(child_dir);
    }
    Ok(())
}

fn extract_archive_file(
    archive_path: &Path,
    archive_type: &str,
    extract_dir: &Path,
) -> Result<(), String> {
    if archive_type == "zip" || archive_path.to_string_lossy().ends_with(".zip") {
        let file = File::open(archive_path).map_err(|e| format!("Failed to open zip archive: {}", e))?;
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
        let file = File::open(archive_path).map_err(|e| format!("Failed to open tar.gz archive: {}", e))?;
        let tar = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(extract_dir).map_err(|e| format!("Failed to unpack tar.gz: {}", e))?;
    } else {
        return Err(format!("Unsupported archive format: {}", archive_type));
    }
    Ok(())
}

async fn download_and_stream(
    client: &reqwest::Client,
    url: &str,
    dest_path: &Path,
    installer_mgr: &RuntimeInstallerManager,
    runtime_id: &str,
    status_label: &str,
    base_percent: f32,
    percent_span: f32,
    message: Option<String>,
) -> Result<u64, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to initiate download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with HTTP status {}: {}", response.status(), url));
    }

    let total_bytes = response.content_length().unwrap_or(0);
    let mut downloaded_bytes: u64 = 0;
    let mut stream = response.bytes_stream();

    let mut file = tokio::fs::File::create(dest_path)
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

            let fraction = if total_bytes > 0 {
                downloaded_bytes as f32 / total_bytes as f32
            } else {
                0.5
            };
            let percent = base_percent + fraction * percent_span;

            installer_mgr.update_progress(InstallProgress {
                runtime_id: runtime_id.to_string(),
                status: status_label.to_string(),
                percent,
                speed_mbps,
                downloaded_bytes,
                total_bytes,
                error_message: None,
                message: message.clone(),
            });
        }
    }

    use tokio::io::AsyncWriteExt;
    file.flush().await.map_err(|e| format!("Failed to flush archive file: {}", e))?;
    Ok(downloaded_bytes)
}

pub fn find_installed_binary(runtimes_dir: &Path, manifest: &RuntimeManifest) -> Option<PathBuf> {
    let target_dir = runtimes_dir.join(&manifest.id);
    if !target_dir.exists() {
        return None;
    }

    let direct_bin = target_dir.join(&manifest.binary_name);
    if direct_bin.exists() && direct_bin.is_file() {
        return Some(direct_bin);
    }

    let direct_bin_exe = target_dir.join(format!("{}.exe", &manifest.binary_name));
    if direct_bin_exe.exists() && direct_bin_exe.is_file() {
        return Some(direct_bin_exe);
    }

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

pub fn uninstall_runtime(runtimes_dir: &Path, runtime_id: &str) -> Result<(), String> {
    let target_dir = runtimes_dir.join(runtime_id);
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).map_err(|e| format!("Failed to remove runtime directory: {}", e))?;
        tracing::info!("Uninstalled runtime '{}' from {}", runtime_id, target_dir.display());
    }
    Ok(())
}

pub async fn install_runtime(
    installer_mgr: Arc<RuntimeInstallerManager>,
    runtimes_dir: PathBuf,
    manifest: &RuntimeManifest,
    hardware_override: Option<String>,
) -> Result<PathBuf, String> {
    let runtime_id = manifest.id.clone();
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

    fs::create_dir_all(&runtimes_dir)
        .map_err(|e| format!("Failed to create runtimes directory: {}", e))?;

    let main_archive_path = runtimes_dir.join(format!(".tmp_main_{}.archive", runtime_id));
    let temp_extract_dir = runtimes_dir.join(format!(".tmp_extract_{}", runtime_id));

    let _ = fs::remove_file(&main_archive_path);
    let _ = fs::remove_dir_all(&temp_extract_dir);

    let num_extras = download_target.extra_archives.len();
    let total_archives = 1 + num_extras;
    let download_span_per_archive = 85.0 / total_archives as f32;

    let main_msg = if total_archives > 1 {
        format!("Downloading engine (1/{})...", total_archives)
    } else {
        "Downloading engine...".to_string()
    };

    installer_mgr.update_progress(InstallProgress {
        runtime_id: runtime_id.clone(),
        status: "downloading".to_string(),
        percent: 0.0,
        speed_mbps: 0.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        error_message: None,
        message: Some(main_msg.clone()),
    });

    let client = reqwest::Client::builder()
        .user_agent("Monolai-Runtime-Manager/1.0")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let mut total_downloaded_bytes: u64 = 0;
    match download_and_stream(
        &client,
        &download_target.url,
        &main_archive_path,
        &installer_mgr,
        &runtime_id,
        "downloading",
        0.0,
        download_span_per_archive,
        Some(main_msg),
    ).await {
        Ok(bytes) => total_downloaded_bytes += bytes,
        Err(err) => {
            let _ = fs::remove_file(&main_archive_path);
            installer_mgr.update_progress(InstallProgress {
                runtime_id: runtime_id.clone(),
                status: "error".to_string(),
                percent: 0.0,
                speed_mbps: 0.0,
                downloaded_bytes: 0,
                total_bytes: 0,
                error_message: Some(err.clone()),
                message: None,
            });
            return Err(err);
        }
    }

    let mut extra_archive_paths = Vec::new();
    for (i, extra) in download_target.extra_archives.iter().enumerate() {
        let extra_path = runtimes_dir.join(format!(".tmp_extra_{}_{}.archive", runtime_id, i));
        let _ = fs::remove_file(&extra_path);
        let base_p = (1 + i) as f32 * download_span_per_archive;
        let extra_msg = format!("Downloading dependencies ({}/{})...", 2 + i, total_archives);

        tracing::info!(
            "Downloading extra dependency for runtime '{}': {}",
            runtime_id,
            extra.url
        );

        installer_mgr.update_progress(InstallProgress {
            runtime_id: runtime_id.clone(),
            status: "downloading".to_string(),
            percent: base_p,
            speed_mbps: 0.0,
            downloaded_bytes: total_downloaded_bytes,
            total_bytes: 0,
            error_message: None,
            message: Some(extra_msg.clone()),
        });

        match download_and_stream(
            &client,
            &extra.url,
            &extra_path,
            &installer_mgr,
            &runtime_id,
            "downloading",
            base_p,
            download_span_per_archive,
            Some(extra_msg),
        ).await {
            Ok(bytes) => {
                total_downloaded_bytes += bytes;
                extra_archive_paths.push((extra_path, extra.archive_type.clone()));
            }
            Err(err) => {
                let _ = fs::remove_file(&main_archive_path);
                for (p, _) in &extra_archive_paths {
                    let _ = fs::remove_file(p);
                }
                let _ = fs::remove_file(&extra_path);
                installer_mgr.update_progress(InstallProgress {
                    runtime_id: runtime_id.clone(),
                    status: "error".to_string(),
                    percent: 0.0,
                    speed_mbps: 0.0,
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    error_message: Some(err.clone()),
                    message: None,
                });
                return Err(err);
            }
        }
    }

    installer_mgr.update_progress(InstallProgress {
        runtime_id: runtime_id.clone(),
        status: "extracting".to_string(),
        percent: 90.0,
        speed_mbps: 0.0,
        downloaded_bytes: total_downloaded_bytes,
        total_bytes: total_downloaded_bytes,
        error_message: None,
        message: Some("Extracting files...".to_string()),
    });

    fs::create_dir_all(&temp_extract_dir)
        .map_err(|e| format!("Failed to create temporary extraction directory: {}", e))?;

    let main_archive_clone = main_archive_path.clone();
    let temp_extract_clone = temp_extract_dir.clone();
    let main_archive_type = download_target.archive_type.clone();

    let extract_result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        extract_archive_file(&main_archive_clone, &main_archive_type, &temp_extract_clone)?;
        let _ = flatten_single_child_dir(&temp_extract_clone);
        Ok(())
    })
    .await
    .map_err(|e| format!("Extraction task panicked: {}", e))?;

    if let Err(e) = extract_result {
        let _ = fs::remove_file(&main_archive_path);
        for (p, _) in &extra_archive_paths {
            let _ = fs::remove_file(p);
        }
        let _ = fs::remove_dir_all(&temp_extract_dir);
        installer_mgr.update_progress(InstallProgress {
            runtime_id: runtime_id.clone(),
            status: "error".to_string(),
            percent: 0.0,
            speed_mbps: 0.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            error_message: Some(e.clone()),
            message: None,
        });
        return Err(e);
    }

    for (extra_path, extra_type) in extra_archive_paths.drain(..) {
        let temp_sub = runtimes_dir.join(format!(".tmp_sub_extra_{}", runtime_id));
        let _ = fs::remove_dir_all(&temp_sub);
        fs::create_dir_all(&temp_sub)
            .map_err(|e| format!("Failed to create temp dependency dir: {}", e))?;

        let extract_sub = tokio::task::spawn_blocking({
            let ep = extra_path.clone();
            let et = extra_type.clone();
            let ts = temp_sub.clone();
            move || -> Result<(), String> {
                extract_archive_file(&ep, &et, &ts)?;
                let _ = flatten_single_child_dir(&ts);
                Ok(())
            }
        })
        .await
        .map_err(|e| format!("Extra extraction task panicked: {}", e))?;

        let _ = fs::remove_file(&extra_path);

        if let Err(e) = extract_sub {
            tracing::warn!("Warning during extra archive extraction: {}", e);
        } else {
            if let Ok(entries) = walkdir::WalkDir::new(&temp_sub).into_iter().collect::<Result<Vec<_>, _>>() {
                for entry in entries {
                    if entry.file_type().is_file() {
                        let rel = entry.path().strip_prefix(&temp_sub).unwrap_or(entry.path());
                        let target_file = temp_extract_dir.join(rel);
                        if let Some(parent) = target_file.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let _ = fs::copy(entry.path(), &target_file);
                    }
                }
            }
        }
        let _ = fs::remove_dir_all(&temp_sub);
    }

    save_installed_metadata(&temp_extract_dir, &runtime_id, &manifest.version, &acceleration);

    let final_dest_dir = runtimes_dir.join(&runtime_id);
    let _ = fs::remove_dir_all(&final_dest_dir);

    let _ = flatten_single_child_dir(&temp_extract_dir);

    fs::rename(&temp_extract_dir, &final_dest_dir)
        .map_err(|e| format!("Failed to move extracted runtime to {}: {}", final_dest_dir.display(), e))?;

    let _ = fs::remove_file(&main_archive_path);
    let _ = fs::remove_dir_all(&temp_extract_dir);

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

    let resolved_binary = find_installed_binary(&runtimes_dir, manifest)
        .ok_or_else(|| format!("Binary '{}' not found after extraction in {}", manifest.binary_name, final_dest_dir.display()))?;

    installer_mgr.update_progress(InstallProgress {
        runtime_id: runtime_id.clone(),
        status: "completed".to_string(),
        percent: 100.0,
        speed_mbps: 0.0,
        downloaded_bytes: total_downloaded_bytes,
        total_bytes: total_downloaded_bytes,
        error_message: None,
        message: Some("Installation completed".to_string()),
    });

    tracing::info!(
        "Runtime '{}' ({}) successfully installed at: {}",
        runtime_id,
        acceleration,
        resolved_binary.display()
    );

    Ok(resolved_binary)
}
