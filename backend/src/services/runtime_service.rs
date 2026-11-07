use crate::core::config::AppConfig;
use crate::core::error::{AppError, AppResult};
use crate::domain::{AccelerationOption, InstallProgress, RuntimeItem, RuntimeManifest};
use crate::infrastructure::downloader::{
    fetch_upstream, find_installed_binary, get_installed_metadata, install_runtime,
    load_root_upstream_metadata, save_root_upstream_metadata, uninstall_runtime,
    RuntimeInstallerManager,
};
use crate::infrastructure::hardware::detect_hardware;
use crate::infrastructure::process::manifests::{get_manifest_for_runtime, get_runtime_manifests};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct RuntimeService {
    installer_manager: Arc<RuntimeInstallerManager>,
    config: Arc<RwLock<AppConfig>>,
    extra_manifests_folder: Option<PathBuf>,
    client: reqwest::Client,
}

impl RuntimeService {
    pub fn new(
        installer_manager: Arc<RuntimeInstallerManager>,
        config: Arc<RwLock<AppConfig>>,
        extra_manifests_folder: Option<PathBuf>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(concat!("monolai/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            installer_manager,
            config,
            extra_manifests_folder,
            client,
        }
    }

    pub fn manifests(&self) -> Vec<RuntimeManifest> {
        get_runtime_manifests(self.extra_manifests_folder.as_deref())
    }

    pub fn manifest_by_id(&self, id: &str) -> Option<RuntimeManifest> {
        get_manifest_for_runtime(self.extra_manifests_folder.as_deref(), id)
    }

    async fn get_runtimes_dir(&self) -> PathBuf {
        let cfg = self.config.read().await;
        let dir = cfg.runtimes.as_deref().unwrap_or("").trim().to_string();
        if dir.is_empty() {
            crate::core::config::get_default_runtimes_dir()
        } else {
            crate::core::config::expand_tilde(dir)
        }
    }

    /// Check upstream sources for all manifests in parallel and update root `{runtimes_dir}/.metadata.json`.
    pub async fn check_updates(&self, force: bool) -> Vec<RuntimeItem> {
        let runtimes_dir = self.get_runtimes_dir().await;

        let mut root_upstreams = load_root_upstream_metadata(&runtimes_dir);
        let manifests = self.manifests();

        let mut tasks = Vec::new();
        for manifest in manifests {
            if let Some(upstream_config) = manifest.upstream {
                let cached = if force {
                    None
                } else {
                    root_upstreams.get(&manifest.id).cloned()
                };
                let client = self.client.clone();
                let manifest_id = manifest.id.clone();

                tasks.push(async move {
                    let res = fetch_upstream(&client, &upstream_config, cached.as_ref()).await;
                    (manifest_id, res)
                });
            }
        }

        let results = futures_util::future::join_all(tasks).await;
        let mut changed = false;

        for (id, res) in results {
            match res {
                Ok(Some(fresh)) => {
                    tracing::info!(
                        "Updated upstream metadata for runtime '{}': version {}",
                        id,
                        fresh.version
                    );
                    root_upstreams.insert(id, fresh);
                    changed = true;
                }
                Ok(None) => {
                    tracing::debug!(
                        "Upstream for runtime '{}' is already up to date (304 Not Modified)",
                        id
                    );
                }
                Err(err) => {
                    tracing::warn!("Failed to check upstream for runtime '{}': {}", id, err);
                }
            }
        }

        if changed {
            if let Err(err) = save_root_upstream_metadata(&runtimes_dir, &root_upstreams) {
                tracing::error!("Failed to save root upstream metadata: {}", err);
            }
        }

        self.build_runtime_items(&runtimes_dir, &root_upstreams)
    }

    pub async fn get_runtimes(&self) -> Vec<RuntimeItem> {
        let runtimes_dir = self.get_runtimes_dir().await;
        let root_upstreams = load_root_upstream_metadata(&runtimes_dir);
        let manifests = self.manifests();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 1. If cache is completely empty on first launch, fetch in parallel so versions appear
        let has_no_cache = manifests
            .iter()
            .any(|m| m.upstream.is_some() && !root_upstreams.contains_key(&m.id));

        if has_no_cache {
            return self.check_updates(false).await;
        }

        // 2. If cache exists but is older than 6 hours (21600s), trigger background refresh (non-blocking)
        let is_stale = manifests.iter().any(|m| {
            if m.upstream.is_some() {
                match root_upstreams.get(&m.id) {
                    None => true,
                    Some(entry) => now.saturating_sub(entry.last_checked_at) > 21600,
                }
            } else {
                false
            }
        });

        if is_stale {
            let self_clone = self.clone();
            tokio::spawn(async move {
                tracing::debug!("Triggering non-blocking background refresh for stale runtime upstreams");
                let _ = self_clone.check_updates(false).await;
            });
        }

        // 3. Return immediately from disk cache! (Instant UI load, 0ms latency)
        self.build_runtime_items(&runtimes_dir, &root_upstreams)
    }

    fn build_runtime_items(
        &self,
        runtimes_dir: &std::path::Path,
        root_upstreams: &std::collections::HashMap<String, crate::domain::runtimes::UpstreamRuntimeEntry>,
    ) -> Vec<RuntimeItem> {
        let hw_report = detect_hardware();
        let active_accel = hw_report.recommended_acceleration.clone();
        let mut manifests = self.manifests();
        let mut items = Vec::new();

        for mut manifest in manifests.drain(..) {
            // Apply root upstream metadata if available
            if let Some(entry) = root_upstreams.get(&manifest.id) {
                manifest.apply_upstream(entry);
            }

            let binary_path = find_installed_binary(runtimes_dir, &manifest);
            let is_installed = binary_path.is_some();
            let installed_path = binary_path.map(|p| p.to_string_lossy().to_string());
            let progress = self.installer_manager.get_progress(&manifest.id);
            let installed_meta = get_installed_metadata(runtimes_dir, &manifest.id);
            let installed_accel = installed_meta.as_ref().map(|m| m.acceleration.clone());
            let installed_version = installed_meta.as_ref().map(|m| m.version.clone());
            let has_update = is_installed
                && installed_version.is_some()
                && !manifest.version.is_empty()
                && installed_version.as_deref() != Some(&manifest.version);

            let avail_keys = manifest.get_available_accelerations(&hw_report.os, &hw_report.arch);
            let mut avail_options = Vec::new();
            let mut recommended_found = false;

            for key in avail_keys {
                let is_rec = if !recommended_found {
                    if key == active_accel
                        || (active_accel == "cuda" && key.starts_with("cuda"))
                        || (active_accel == "metal" && key == "metal")
                        || (active_accel == "rocm" && key.starts_with("rocm"))
                        || (active_accel == "sycl" && key.starts_with("sycl"))
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
                if let Some(vulkan_opt) = avail_options.iter_mut().find(|o| {
                    o.id == "vulkan"
                        && hw_report
                            .available_accelerations
                            .contains(&"vulkan".to_string())
                }) {
                    vulkan_opt.is_recommended = true;
                } else if let Some(cpu_opt) = avail_options.iter_mut().find(|o| o.id == "cpu") {
                    cpu_opt.is_recommended = true;
                } else {
                    avail_options[0].is_recommended = true;
                }
            }

            let release_notes_url = root_upstreams
                .get(&manifest.id)
                .and_then(|u| u.release_notes_url.clone());

            let runtime_dir = runtimes_dir.join(&manifest.id);
            let platform_key =
                crate::infrastructure::process::dependencies::detect_current_platform_key();
            let mut dep_statuses = Vec::new();
            for dep in &manifest.system_dependencies {
                let satisfied =
                    crate::infrastructure::process::dependencies::check_system_dependency_satisfied(
                        dep,
                        if is_installed {
                            Some(&runtime_dir)
                        } else {
                            None
                        },
                    );
                let install_cmd =
                    crate::infrastructure::process::dependencies::resolve_install_command(
                        &dep.instructions,
                        &platform_key,
                    );
                dep_statuses.push(crate::domain::runtimes::SystemDependencyStatus {
                    id: dep.id.clone(),
                    name: dep.name.clone(),
                    description: dep.description.clone(),
                    is_satisfied: satisfied,
                    install_command: install_cmd,
                });
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
                installed_version,
                has_update,
                active_acceleration: active_accel.clone(),
                installed_acceleration: installed_accel,
                available_accelerations: avail_options,
                install_progress: progress,
                release_notes_url,
                system_dependencies: dep_statuses,
            });
        }

        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }

    pub async fn install(
        &self,
        runtime_id: &str,
        hardware_override: Option<String>,
    ) -> AppResult<()> {
        let runtimes_dir = self.get_runtimes_dir().await;

        let mut manifest = self.manifest_by_id(runtime_id).ok_or_else(|| {
            AppError::not_found(format!("Runtime manifest for '{}' not found", runtime_id))
        })?;

        // Enrich manifest with resolved upstream variables from root .metadata.json
        let mut root_upstreams = load_root_upstream_metadata(&runtimes_dir);
        let needs_fetch = match root_upstreams.get(runtime_id) {
            Some(entry) => entry.assets.is_empty() && manifest.upstream.is_some(),
            None => manifest.upstream.is_some(),
        };

        if needs_fetch {
            if let Some(upstream_config) = &manifest.upstream {
                match fetch_upstream(&self.client, upstream_config, None).await {
                    Ok(Some(fresh)) => {
                        manifest.apply_upstream(&fresh);
                        root_upstreams.insert(runtime_id.to_string(), fresh);
                        let _ = save_root_upstream_metadata(&runtimes_dir, &root_upstreams);
                    }
                    Ok(None) => {
                        if let Some(entry) = root_upstreams.get(runtime_id) {
                            manifest.apply_upstream(entry);
                        }
                    }
                    Err(err) => {
                        tracing::warn!(
                            "Could not fetch upstream before installing runtime '{}': {}",
                            runtime_id,
                            err
                        );
                        if let Some(entry) = root_upstreams.get(runtime_id) {
                            manifest.apply_upstream(entry);
                        }
                    }
                }
            }
        } else if let Some(entry) = root_upstreams.get(runtime_id) {
            manifest.apply_upstream(entry);
        }

        let hw = hardware_override;
        let installer_mgr = self.installer_manager.clone();
        let rid = runtime_id.to_string();

        tracing::info!(
            target: "monolai",
            "Initiated installation for runtime '{}' (hardware: {:?})",
            rid,
            hw
        );

        tokio::spawn(async move {
            tracing::info!(target: "monolai", "Started download and installation for runtime '{}'", rid);
            if let Err(err) = install_runtime(installer_mgr, runtimes_dir, &manifest, hw).await {
                tracing::error!(target: "monolai", "Failed to install runtime '{}': {}", rid, err);
            } else {
                tracing::info!(target: "monolai", "Successfully installed runtime '{}'", rid);
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
        let runtimes_dir = self.get_runtimes_dir().await;

        tracing::info!(target: "monolai", "Uninstalling runtime '{}'...", runtime_id);
        uninstall_runtime(&runtimes_dir, runtime_id).map_err(|e| {
            tracing::error!(target: "monolai", "Failed to uninstall runtime '{}': {}", runtime_id, e);
            AppError::internal(e)
        })?;
        tracing::info!(target: "monolai", "Successfully uninstalled runtime '{}'", runtime_id);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::downloader::RuntimeInstallerManager;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_runtime_recommendation_never_defaults_to_cuda_without_gpu() {
        let mut cfg = crate::core::config::AppConfig::default();
        cfg.runtimes = Some("/tmp".to_string());

        let service = RuntimeService::new(
            Arc::new(RuntimeInstallerManager::new()),
            Arc::new(RwLock::new(cfg)),
            None,
        );

        let items = service.get_runtimes().await;
        let llama = items
            .iter()
            .find(|i| i.id == "llama-cpp")
            .expect("llama-cpp should exist");

        let rec = llama
            .available_accelerations
            .iter()
            .find(|a| a.is_recommended);
        assert!(rec.is_some(), "There must be a recommended acceleration");

        let hw = crate::infrastructure::hardware::tracker::detect_hardware();
        if !hw.available_accelerations.contains(&"cuda".to_string()) {
            assert_ne!(
                rec.unwrap().id,
                "cuda-12.8",
                "Non-CUDA machine must NOT have cuda-12.8 recommended"
            );
            assert!(
                rec.unwrap().id == "vulkan" || rec.unwrap().id == "cpu",
                "Expected vulkan or cpu, got {}",
                rec.unwrap().id
            );
        }
    }

    #[tokio::test]
    async fn test_get_runtimes_creates_root_metadata_and_populates_versions() {
        let temp_dir = std::env::temp_dir().join(format!(
            "monolai_test_rt_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut cfg = crate::core::config::AppConfig::default();
        cfg.runtimes = Some(temp_dir.to_string_lossy().to_string());

        let service = RuntimeService::new(
            Arc::new(RuntimeInstallerManager::new()),
            Arc::new(RwLock::new(cfg)),
            None,
        );

        let items = service.get_runtimes().await;
        assert!(!items.is_empty());
        let llama = items
            .iter()
            .find(|i| i.id == "llama-cpp")
            .expect("llama-cpp must exist");
        assert!(
            !llama.version.is_empty(),
            "llama-cpp version must be populated from upstream"
        );

        let meta_file = temp_dir.join(".metadata.json");
        assert!(
            meta_file.exists(),
            ".metadata.json must be created in runtimes root"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
