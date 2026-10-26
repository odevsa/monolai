use crate::core::config::AppConfig;
use crate::core::error::{AppError, AppResult};
use crate::domain::{DownloadRecord, EnqueueDownloadPayload, ModelPreset, ModelRecord};
use crate::infrastructure::db::{DownloadRepository, ModelRepository, RepositoryRepository};
use crate::infrastructure::scanner::gguf::inspect_model_file;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::{broadcast, Mutex, RwLock};

struct ActiveTask {
    cancel_tx: tokio::sync::watch::Sender<bool>,
    pause_tx: tokio::sync::watch::Sender<bool>,
    speed_bytes_sec: Arc<Mutex<u64>>,
}

#[derive(Clone)]
pub struct DownloadService {
    download_repo: DownloadRepository,
    model_repo: ModelRepository,
    repo_repo: RepositoryRepository,
    config: Arc<RwLock<AppConfig>>,
    active_tasks: Arc<Mutex<HashMap<String, ActiveTask>>>,
    broadcast_tx: broadcast::Sender<Vec<DownloadRecord>>,
}

impl DownloadService {
    pub fn new(
        download_repo: DownloadRepository,
        model_repo: ModelRepository,
        repo_repo: RepositoryRepository,
        config: Arc<RwLock<AppConfig>>,
    ) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            download_repo,
            model_repo,
            repo_repo,
            config,
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
        }
    }

    async fn resolve_models_dir(&self) -> PathBuf {
        let cfg = self.config.read().await;
        if let Some(ref dir) = cfg.models {
            crate::core::config::expand_tilde(dir)
        } else {
            crate::core::config::get_default_models_dir()
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Vec<DownloadRecord>> {
        self.broadcast_tx.subscribe()
    }

    pub async fn get_all(&self) -> AppResult<Vec<DownloadRecord>> {
        let mut list = self.download_repo.get_all().await?;
        let active = self.active_tasks.lock().await;

        for item in &mut list {
            if let Some(task) = active.get(&item.id) {
                item.speed_bytes_sec = *task.speed_bytes_sec.lock().await;
            }
        }

        Ok(list)
    }

    pub async fn notify_broadcast(&self) {
        if let Ok(records) = self.get_all().await {
            let _ = self.broadcast_tx.send(records);
        }
    }

    pub async fn enqueue_download(
        &self,
        payload: EnqueueDownloadPayload,
    ) -> AppResult<DownloadRecord> {
        let url = if let Some(raw_url) = payload
            .url
            .as_deref()
            .map(str::trim)
            .filter(|u| !u.is_empty())
        {
            raw_url.to_string()
        } else {
            let rid = payload.repo_id.as_deref().unwrap_or("huggingface");
            if let Ok(Some(repo)) = self.repo_repo.get_by_id(rid).await {
                let base = repo.get_mirror_url().unwrap_or(repo.base_url);
                format!(
                    "{}/{}/resolve/main/{}",
                    base.trim_end_matches('/'),
                    payload.model_name.trim(),
                    payload.filename.trim()
                )
            } else {
                format!(
                    "https://huggingface.co/{}/resolve/main/{}",
                    payload.model_name.trim(),
                    payload.filename.trim()
                )
            }
        };

        if url.is_empty() {
            return Err(AppError::bad_request("Download URL cannot be empty."));
        }

        let mut filename = payload.filename.trim().to_string();
        if filename.is_empty() {
            if let Some(last_seg) = url.split('/').last() {
                filename = last_seg
                    .split('?')
                    .next()
                    .unwrap_or("model.gguf")
                    .to_string();
            } else {
                filename = "model.gguf".to_string();
            }
        }

        let modality = payload
            .modality
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| "text-to-text".to_string());

        let model_name = payload.model_name.trim();
        let models_dir = self.resolve_models_dir().await;

        // Canonical directory structure: {type}/{user}/{model}/{filename}
        // e.g. text-to-text/Qwen/Qwen2.5-1.5B-Instruct-GGUF/qwen2.5-1.5b-instruct-q4_k_m.gguf
        let relative_path = if !model_name.is_empty() {
            let parts: Vec<&str> = model_name.trim_matches('/').split('/').collect();
            if parts.len() >= 2 {
                let user = parts[0].trim();
                let model = parts[1..].join("/").trim().to_string();
                format!("{}/{}/{}/{}", modality, user, model, filename)
            } else {
                format!("{}/{}/{}", modality, model_name, filename)
            }
        } else {
            format!("{}/{}", modality, filename)
        };

        let absolute_path = models_dir.join(&relative_path);
        if let Some(parent) = absolute_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        let id = uuid::Uuid::new_v4().to_string();
        let record = DownloadRecord {
            id: id.clone(),
            model_name: model_name.to_string(),
            url: url.clone(),
            file_path: relative_path,
            total_bytes: 0,
            downloaded_bytes: 0,
            status: "queued".to_string(),
            error_message: None,
            created_at: String::new(),
            updated_at: String::new(),
            speed_bytes_sec: 0,
        };

        self.download_repo.insert(&record).await?;
        self.notify_broadcast().await;

        // Spawn background worker
        self.spawn_download_task(
            id.clone(),
            absolute_path,
            url,
            payload.repo_id,
            Some(model_name.to_string()),
        )
        .await;

        self.download_repo
            .get_by_id(&id)
            .await?
            .ok_or_else(|| AppError::internal("Failed to retrieve created download record"))
    }

    async fn spawn_download_task(
        &self,
        task_id: String,
        target_path: PathBuf,
        url: String,
        repo_id: Option<String>,
        model_name: Option<String>,
    ) {
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
        let (pause_tx, pause_rx) = tokio::sync::watch::channel(false);
        let speed_bytes_sec = Arc::new(Mutex::new(0u64));

        {
            let mut active = self.active_tasks.lock().await;
            active.insert(
                task_id.clone(),
                ActiveTask {
                    cancel_tx,
                    pause_tx,
                    speed_bytes_sec: speed_bytes_sec.clone(),
                },
            );
        }

        let this = self.clone();
        tokio::spawn(async move {
            this.run_download_loop(
                task_id.clone(),
                target_path,
                url,
                repo_id,
                model_name,
                cancel_rx,
                pause_rx,
                speed_bytes_sec,
            )
            .await;

            {
                let mut active = this.active_tasks.lock().await;
                active.remove(&task_id);
            }
            this.notify_broadcast().await;
        });
    }

    async fn run_download_loop(
        &self,
        task_id: String,
        target_path: PathBuf,
        url: String,
        repo_id: Option<String>,
        model_name: Option<String>,
        mut cancel_rx: tokio::sync::watch::Receiver<bool>,
        mut pause_rx: tokio::sync::watch::Receiver<bool>,
        speed_ref: Arc<Mutex<u64>>,
    ) {
        let _ = self
            .download_repo
            .update_status(&task_id, "downloading", None)
            .await;
        self.notify_broadcast().await;

        let part_path = PathBuf::from(format!("{}.part", target_path.display()));

        // Inspect existing partial file size for HTTP Range resume
        let mut existing_bytes = 0u64;
        if let Ok(metadata) = tokio::fs::metadata(&part_path).await {
            existing_bytes = metadata.len();
        }

        // Fetch optional repository auth token
        let mut auth_token = None;
        if let Some(ref rid) = repo_id {
            if let Ok(Some(repo)) = self.repo_repo.get_by_id(rid).await {
                auth_token = repo.api_token;
            }
        }

        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .read_timeout(Duration::from_secs(60))
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(30))
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .user_agent("Monolai/0.2.3")
            .build()
            .unwrap_or_default();

        let mut req = client.get(&url);
        if let Some(ref tok) = auth_token {
            if !tok.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", tok));
            }
        }

        if existing_bytes > 0 {
            req = req.header("Range", format!("bytes={}-", existing_bytes));
        }

        let resp = tokio::select! {
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() {
                    let _ = tokio::fs::remove_file(&part_path).await;
                    let _ = self.download_repo.update_status(&task_id, "cancelled", None).await;
                    return;
                }
                return;
            }
            res = req.send() => match res {
                Ok(r) => r,
                Err(e) => {
                    let err_msg = format!("Network error connecting to model URL: {}", e);
                    let _ = self.download_repo.update_status(&task_id, "failed", Some(&err_msg)).await;
                    return;
                }
            }
        };

        let status = resp.status();
        if !status.is_success() && status.as_u16() != 206 {
            let err_msg = if status.as_u16() == 401 {
                "Authentication required (401 Unauthorized). Please configure an API token for this repository in Settings.".to_string()
            } else {
                format!("HTTP server returned error: {}", status)
            };
            let _ = self
                .download_repo
                .update_status(&task_id, "failed", Some(&err_msg))
                .await;
            return;
        }

        let is_range_accepted = status.as_u16() == 206;
        let mut downloaded_bytes = if is_range_accepted { existing_bytes } else { 0 };

        let content_length = resp.content_length().unwrap_or(0);
        let total_bytes = if is_range_accepted {
            existing_bytes + content_length
        } else {
            content_length
        };

        let file_open_res = if is_range_accepted && existing_bytes > 0 {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&part_path)
                .await
        } else {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&part_path)
                .await
        };

        let mut file = match file_open_res {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("Failed to create or open file on disk: {}", e);
                let _ = self
                    .download_repo
                    .update_status(&task_id, "failed", Some(&err_msg))
                    .await;
                return;
            }
        };

        let mut stream = resp.bytes_stream();
        let mut last_sample_time = Instant::now();
        let mut bytes_since_sample = 0u64;
        let mut last_db_update = Instant::now();

        loop {
            let chunk_opt = tokio::select! {
                _ = cancel_rx.changed() => {
                    if *cancel_rx.borrow() {
                        let _ = tokio::fs::remove_file(&part_path).await;
                        let _ = self.download_repo.update_status(&task_id, "cancelled", None).await;
                        return;
                    }
                    None
                }
                _ = pause_rx.changed() => {
                    if *pause_rx.borrow() {
                        let _ = self.download_repo.update_status(&task_id, "paused", None).await;
                        return;
                    }
                    None
                }
                chunk_res = stream.next() => chunk_res,
            };

            let chunk_res = match chunk_opt {
                Some(c) => c,
                None => break,
            };

            let chunk = match chunk_res {
                Ok(c) => c,
                Err(e) => {
                    let err_msg = format!("Error streaming chunk: {}", e);
                    let _ = self
                        .download_repo
                        .update_status(&task_id, "failed", Some(&err_msg))
                        .await;
                    return;
                }
            };

            if let Err(e) = file.write_all(&chunk).await {
                let err_msg = format!("Failed to write chunk to disk: {}", e);
                let _ = self
                    .download_repo
                    .update_status(&task_id, "failed", Some(&err_msg))
                    .await;
                return;
            }

            let len = chunk.len() as u64;
            downloaded_bytes += len;
            bytes_since_sample += len;

            // Speed calculation every 500ms
            if last_sample_time.elapsed() >= Duration::from_millis(500) {
                let elapsed_secs = last_sample_time.elapsed().as_secs_f64();
                let current_speed = (bytes_since_sample as f64 / elapsed_secs) as u64;
                *speed_ref.lock().await = current_speed;
                last_sample_time = Instant::now();
                bytes_since_sample = 0;
            }

            // DB progress & SSE broadcast update every 1 second
            if last_db_update.elapsed() >= Duration::from_secs(1) {
                let _ = self
                    .download_repo
                    .update_progress(&task_id, downloaded_bytes as i64, total_bytes as i64)
                    .await;
                self.notify_broadcast().await;
                last_db_update = Instant::now();
            }
        }

        let _ = file.flush().await;
        drop(file);

        // Rename .part to target path
        if let Err(e) = tokio::fs::rename(&part_path, &target_path).await {
            let err_msg = format!("Failed to finalize downloaded file: {}", e);
            let _ = self
                .download_repo
                .update_status(&task_id, "failed", Some(&err_msg))
                .await;
            return;
        }

        *speed_ref.lock().await = 0;
        let _ = self
            .download_repo
            .update_progress(&task_id, downloaded_bytes as i64, downloaded_bytes as i64)
            .await;
        let _ = self
            .download_repo
            .update_status(&task_id, "completed", None)
            .await;

        // Auto-register model in DB and create default preset
        self.auto_register_completed_model(&target_path, repo_id.as_deref(), model_name.as_deref())
            .await;
        self.notify_broadcast().await;
    }

    async fn auto_register_completed_model(
        &self,
        target_path: &Path,
        _repo_id: Option<&str>,
        model_name: Option<&str>,
    ) {
        let models_dir = self.resolve_models_dir().await;
        let filename = target_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let relative_path = target_path
            .strip_prefix(&models_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| filename.clone());

        let inspect = inspect_model_file(target_path);

        let norm_path = relative_path.replace('\\', "/");
        let parts: Vec<&str> = norm_path.split('/').collect();

        // Canonical ID pattern: {user}/{model}:{filename}
        let (id, display_name, source_repo) =
            if let Some(mn) = model_name.filter(|s| !s.trim().is_empty()) {
                let clean_mn = mn.trim().trim_matches('/');
                (
                    format!("{}:{}", clean_mn, filename),
                    clean_mn.to_string(),
                    Some(clean_mn.to_string()),
                )
            } else if parts.len() >= 4 {
                let user = parts[1];
                let repo = parts[2];
                let repo_slug = format!("{}/{}", user, repo);
                (
                    format!("{}/{}:{}", user, repo, filename),
                    repo_slug.clone(),
                    Some(repo_slug),
                )
            } else if parts.len() == 3 {
                let folder = parts[1];
                (
                    format!("{}:{}", folder, filename),
                    folder.to_string(),
                    Some(folder.to_string()),
                )
            } else {
                (filename.clone(), filename.clone(), None)
            };

        let metadata = tokio::fs::metadata(target_path).await.ok();
        let size_bytes = metadata.map(|m| m.len()).unwrap_or(0) as i64;

        let model_record = ModelRecord {
            id: id.clone(),
            name: display_name,
            filename,
            relative_path: relative_path.clone(),
            size_bytes,
            format: inspect.format.clone(),
            modality: inspect.modality.clone(),
            architecture: inspect.architecture,
            source_repo,
            created_at: String::new(),
            presets: Vec::new(),
            file_exists: true,
        };

        if self
            .model_repo
            .get_by_id(&id)
            .await
            .unwrap_or(None)
            .is_none()
        {
            let _ = self.model_repo.insert_model(&model_record).await;

            let runtime_id = if inspect.modality == "text-to-image" {
                "sd-cpp"
            } else {
                "llama-cpp"
            };

            let default_preset = ModelPreset {
                id: format!("{}:default", id),
                model_id: id,
                name: "Default".to_string(),
                runtime_id: runtime_id.to_string(),
                flags: serde_json::json!({ "--model": relative_path }).to_string(),
                is_default: true,
                created_at: String::new(),
            };

            let _ = self.model_repo.insert_preset(&default_preset).await;
        }
    }

    pub async fn pause_download(&self, id: &str) -> AppResult<()> {
        let found = {
            let active = self.active_tasks.lock().await;
            if let Some(task) = active.get(id) {
                let _ = task.pause_tx.send(true);
                true
            } else {
                false
            }
        };

        if !found {
            let _ = self.download_repo.update_status(id, "paused", None).await;
        }
        self.notify_broadcast().await;
        Ok(())
    }

    pub async fn resume_download(&self, id: &str) -> AppResult<()> {
        let record = self
            .download_repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found(format!("Download '{}' not found", id)))?;

        let models_dir = self.resolve_models_dir().await;
        let target_path = models_dir.join(&record.file_path);

        let repo_id = if record.url.contains("huggingface.co") {
            Some("huggingface".to_string())
        } else {
            None
        };

        self.spawn_download_task(
            id.to_string(),
            target_path,
            record.url,
            repo_id,
            Some(record.model_name),
        )
        .await;
        Ok(())
    }

    pub async fn cancel_download(&self, id: &str) -> AppResult<()> {
        let found = {
            let active = self.active_tasks.lock().await;
            if let Some(task) = active.get(id) {
                let _ = task.cancel_tx.send(true);
                true
            } else {
                false
            }
        };

        if !found {
            let record = self.download_repo.get_by_id(id).await?;
            if let Some(r) = record {
                let models_dir = self.resolve_models_dir().await;
                let part_path =
                    PathBuf::from(format!("{}/{}.part", models_dir.display(), r.file_path));
                let _ = tokio::fs::remove_file(&part_path).await;
            }
            let _ = self
                .download_repo
                .update_status(id, "cancelled", None)
                .await;
        }
        self.notify_broadcast().await;
        Ok(())
    }

    pub async fn delete_download(&self, id: &str) -> AppResult<()> {
        let _ = self.cancel_download(id).await;
        self.download_repo.delete(id).await?;
        self.notify_broadcast().await;
        Ok(())
    }

    pub async fn clear_inactive_downloads(&self) -> AppResult<u64> {
        let count = self.download_repo.delete_inactive().await?;
        self.notify_broadcast().await;
        Ok(count)
    }
}
