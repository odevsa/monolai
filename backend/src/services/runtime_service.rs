use crate::core::config::AppConfig;
use crate::core::error::{AppError, AppResult};
use crate::domain::{AccelerationOption, InstallProgress, RuntimeItem, RuntimeManifest};
use crate::infrastructure::downloader::{
    find_installed_binary, get_installed_acceleration, install_runtime, uninstall_runtime,
    RuntimeInstallerManager,
};
use crate::infrastructure::hardware::{detect_hardware, resolve_target_acceleration};
use crate::infrastructure::process::manifests::{get_manifest_for_runtime, get_runtime_manifests};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct RuntimeService {
    installer_manager: Arc<RuntimeInstallerManager>,
    config: Arc<RwLock<AppConfig>>,
    extra_manifests_folder: Option<PathBuf>,
}

impl RuntimeService {
    pub fn new(
        installer_manager: Arc<RuntimeInstallerManager>,
        config: Arc<RwLock<AppConfig>>,
        extra_manifests_folder: Option<PathBuf>,
    ) -> Self {
        Self {
            installer_manager,
            config,
            extra_manifests_folder,
        }
    }

    pub fn manifests(&self) -> Vec<RuntimeManifest> {
        get_runtime_manifests(self.extra_manifests_folder.as_deref())
    }

    pub fn manifest_by_id(&self, id: &str) -> Option<RuntimeManifest> {
        get_manifest_for_runtime(self.extra_manifests_folder.as_deref(), id)
    }

    pub async fn get_runtimes(&self) -> Vec<RuntimeItem> {
        let (runtimes_dir, configured_hardware) = {
            let cfg = self.config.read().await;
            let dir = cfg.runtimes.as_deref().unwrap_or("").to_string();
            let hw = cfg.hardware.clone();
            (crate::core::config::expand_tilde(dir), hw)
        };

        let hw_report = detect_hardware();
        let active_accel = resolve_target_acceleration(configured_hardware.as_deref());
        let manifests = self.manifests();
        let mut items = Vec::new();

        for manifest in manifests {
            let binary_path = find_installed_binary(&runtimes_dir, &manifest);
            let is_installed = binary_path.is_some();
            let installed_path = binary_path.map(|p| p.to_string_lossy().to_string());
            let progress = self.installer_manager.get_progress(&manifest.id);
            let installed_accel = get_installed_acceleration(&runtimes_dir, &manifest.id);

            let avail_keys = manifest.get_available_accelerations(&hw_report.os, &hw_report.arch);
            let mut avail_options = Vec::new();
            let mut recommended_found = false;

            for key in avail_keys {
                let is_rec = if !recommended_found {
                    if key == active_accel
                        || (active_accel == "cuda" && key.starts_with("cuda"))
                        || (active_accel == "metal" && key == "metal")
                    {
                        recommended_found = true;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                let label = crate::domain::runtimes::format_acceleration_label(&key);
                avail_options.push(AccelerationOption {
                    id: key,
                    label,
                    is_recommended: is_rec,
                });
            }

            if !recommended_found && !avail_options.is_empty() {
                avail_options[0].is_recommended = true;
            }

            items.push(RuntimeItem {
                id: manifest.id,
                name: manifest.name,
                version: manifest.version,
                icon: manifest.icon,
                website: manifest.website,
                description: manifest.description,
                features: manifest.features,
                is_installed,
                installed_path,
                active_acceleration: active_accel.clone(),
                installed_acceleration: installed_accel,
                available_accelerations: avail_options,
                install_progress: progress,
            });
        }

        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }

    pub async fn install(&self, runtime_id: &str, hardware_override: Option<String>) -> AppResult<()> {
        let (runtimes_dir, configured_hardware) = {
            let cfg = self.config.read().await;
            let dir = cfg.runtimes.as_deref().unwrap_or("").to_string();
            if dir.trim().is_empty() {
                return Err(AppError::bad_request("Runtimes directory is not configured in config.yaml"));
            }
            let hw = cfg.hardware.clone();
            (crate::core::config::expand_tilde(dir), hw)
        };

        let manifest = self
            .manifest_by_id(runtime_id)
            .ok_or_else(|| AppError::not_found(format!("Runtime manifest for '{}' not found", runtime_id)))?;

        let hw = hardware_override.or(configured_hardware);
        let installer_mgr = self.installer_manager.clone();
        let rid = runtime_id.to_string();

        tokio::spawn(async move {
            if let Err(err) = install_runtime(installer_mgr, runtimes_dir, &manifest, hw).await {
                tracing::error!("Failed to install runtime '{}': {}", rid, err);
            }
        });

        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<InstallProgress> {
        self.installer_manager.subscribe()
    }

    pub fn get_progress(&self, runtime_id: &str) -> Option<InstallProgress> {
        self.installer_manager.get_progress(runtime_id)
    }

    pub async fn uninstall(&self, runtime_id: &str) -> AppResult<()> {
        let runtimes_dir = {
            let cfg = self.config.read().await;
            let dir = cfg.runtimes.as_deref().unwrap_or("").to_string();
            if dir.trim().is_empty() {
                return Err(AppError::bad_request("Runtimes directory is not configured in config.yaml"));
            }
            crate::core::config::expand_tilde(dir)
        };

        uninstall_runtime(&runtimes_dir, runtime_id).map_err(AppError::internal)?;

        self.installer_manager.update_progress(InstallProgress {
            runtime_id: runtime_id.to_string(),
            status: "idle".to_string(),
            percent: 0.0,
            speed_mbps: 0.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            error_message: None,
            message: None,
        });

        Ok(())
    }
}
