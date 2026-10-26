use crate::domain::{LogEntry, LogFilterQuery};
use crate::infrastructure::db::SettingRepository;
use std::collections::{BTreeSet, VecDeque};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

const MAX_IN_MEMORY_LOGS: usize = 2000;
const DEFAULT_MAX_LOG_SIZE_MB: u64 = 10;

#[derive(Clone)]
pub struct LogService {
    logs_dir: PathBuf,
    buffer: Arc<RwLock<VecDeque<LogEntry>>>,
    broadcast_sender: broadcast::Sender<LogEntry>,
    next_id: Arc<AtomicU64>,
    setting_repo: SettingRepository,
    known_services: Arc<RwLock<BTreeSet<String>>>,
}

fn sanitize_service_filename(service: &str) -> String {
    let sanitized: String = service
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "service".to_string()
    } else {
        sanitized
    }
}

impl LogService {
    pub fn new(logs_dir: PathBuf, setting_repo: SettingRepository) -> Self {
        let _ = std::fs::create_dir_all(&logs_dir);
        let (broadcast_sender, _) = broadcast::channel(1024);

        let mut initial_services = BTreeSet::new();
        initial_services.insert("monolai".to_string());

        Self {
            logs_dir,
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_IN_MEMORY_LOGS))),
            broadcast_sender,
            next_id: Arc::new(AtomicU64::new(1)),
            setting_repo,
            known_services: Arc::new(RwLock::new(initial_services)),
        }
    }

    /// Checks whether logging is enabled via settings (feature flag "enable_log")
    pub async fn is_logging_enabled(&self) -> bool {
        let val = self.setting_repo.get("enable_log").await.ok().flatten();
        match val {
            Some(s) => {
                let lower = s.trim().to_lowercase();
                lower != "false" && lower != "0" && lower != "off" && lower != "no"
            }
            None => true,
        }
    }

    /// Records a new log entry, stores in memory ring-buffer, writes to disk with rotation, and broadcasts to live listeners
    pub async fn record(&self, service: &str, level: &str, message: &str) {
        if !self.is_logging_enabled().await {
            return;
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let timestamp = chrono::Utc::now().to_rfc3339();

        let entry = LogEntry {
            id,
            timestamp,
            level: level.to_uppercase(),
            service: service.to_string(),
            message: message.to_string(),
        };

        // 1. Add to in-memory buffer
        {
            let mut buf = self.buffer.write().await;
            if buf.len() >= MAX_IN_MEMORY_LOGS {
                buf.pop_front();
            }
            buf.push_back(entry.clone());
        }

        // 2. Track known services
        {
            let mut services = self.known_services.write().await;
            services.insert(service.to_string());
        }

        // 3. Broadcast to real-time SSE subscribers
        let _ = self.broadcast_sender.send(entry.clone());

        // 4. Append to service log file on disk with rotation check
        let logs_dir = self.logs_dir.clone();
        let service_name = service.to_string();
        let formatted_line = format!(
            "[{}] [{}] [{}] {}\n",
            entry.timestamp, entry.level, entry.service, entry.message
        );
        let max_mb = self
            .setting_repo
            .get_parsed("max_log_size_mb", DEFAULT_MAX_LOG_SIZE_MB)
            .await;

        tokio::task::spawn_blocking(move || {
            let file_name = format!("{}.log", sanitize_service_filename(&service_name));
            let file_path = logs_dir.join(&file_name);

            // Check file size for rotation
            if let Ok(meta) = std::fs::metadata(&file_path) {
                let max_bytes = max_mb * 1024 * 1024;
                if meta.len() >= max_bytes {
                    let rotated_path = logs_dir.join(format!("{}.1", file_name));
                    let _ = std::fs::remove_file(&rotated_path);
                    let _ = std::fs::rename(&file_path, &rotated_path);
                }
            }

            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
            {
                let _ = file.write_all(formatted_line.as_bytes());
            }
        });
    }

    /// Query in-memory log entries with filtering and limit
    pub async fn get_entries(&self, filter: &LogFilterQuery) -> Vec<LogEntry> {
        let buf = self.buffer.read().await;
        let limit = filter.limit.unwrap_or(500).min(MAX_IN_MEMORY_LOGS);

        let service_filter = filter
            .service
            .as_deref()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty() && s != "all");

        let level_filter = filter
            .level
            .as_deref()
            .map(|l| l.trim().to_uppercase())
            .filter(|l| !l.is_empty() && l != "ALL");

        let search_filter = filter
            .search
            .as_deref()
            .map(|q| q.trim().to_lowercase())
            .filter(|q| !q.is_empty());

        buf.iter()
            .rev()
            .filter(|entry| {
                if let Some(ref target_svc) = service_filter {
                    if !entry.service.to_lowercase().contains(target_svc) {
                        return false;
                    }
                }
                if let Some(ref target_lvl) = level_filter {
                    if entry.level != *target_lvl {
                        return false;
                    }
                }
                if let Some(ref query) = search_filter {
                    let in_msg = entry.message.to_lowercase().contains(query);
                    let in_svc = entry.service.to_lowercase().contains(query);
                    if !in_msg && !in_svc {
                        return false;
                    }
                }
                true
            })
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// List all known services that have emitted logs
    pub async fn get_services(&self) -> Vec<String> {
        let services = self.known_services.read().await;
        let mut list = Vec::new();
        list.push("all".to_string());
        for svc in services.iter() {
            list.push(svc.clone());
        }
        list
    }

    /// Clears logs from in-memory buffer and removes corresponding log files on disk
    pub async fn clear_logs(&self, service: Option<&str>) {
        let is_all = service.is_none()
            || service.map(|s| s.trim().to_lowercase()) == Some("all".to_string())
            || service.map(|s| s.trim().is_empty()) == Some(true);

        if is_all {
            {
                let mut buf = self.buffer.write().await;
                buf.clear();
            }
            {
                let mut services = self.known_services.write().await;
                services.clear();
                services.insert("monolai".to_string());
            }
            let logs_dir = self.logs_dir.clone();
            let _ = tokio::task::spawn_blocking(move || {
                if let Ok(entries) = std::fs::read_dir(logs_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let _ = std::fs::remove_file(path);
                        }
                    }
                }
            })
            .await;
        } else if let Some(svc) = service {
            let svc_lower = svc.to_lowercase();
            {
                let mut buf = self.buffer.write().await;
                buf.retain(|entry| !entry.service.to_lowercase().contains(&svc_lower));
            }
            {
                let mut services = self.known_services.write().await;
                services.retain(|s| !s.to_lowercase().contains(&svc_lower));
                if services.is_empty() {
                    services.insert("monolai".to_string());
                }
            }
            let file_name = format!("{}.log", sanitize_service_filename(svc));
            let file_path = self.logs_dir.join(&file_name);
            let rotated_path = self.logs_dir.join(format!("{}.1", file_name));
            let _ = tokio::task::spawn_blocking(move || {
                let _ = std::fs::remove_file(file_path);
                let _ = std::fs::remove_file(rotated_path);
            })
            .await;
        }
    }

    /// Reads log content from file for export / download
    pub async fn get_download_content(&self, service: Option<&str>) -> (String, Vec<u8>) {
        let is_all = service.is_none()
            || service.map(|s| s.trim().to_lowercase()) == Some("all".to_string())
            || service.map(|s| s.trim().is_empty()) == Some(true);

        let logs_dir = self.logs_dir.clone();
        let svc_opt = service.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            if is_all {
                let mut combined = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&logs_dir) {
                    let mut file_paths: Vec<PathBuf> = entries
                        .flatten()
                        .map(|e| e.path())
                        .filter(|p| p.extension().map_or(false, |ext| ext == "log"))
                        .collect();
                    file_paths.sort();

                    for p in file_paths {
                        let name = p.file_name().unwrap_or_default().to_string_lossy();
                        let header = format!("=== Log file: {} ===\n", name);
                        combined.extend_from_slice(header.as_bytes());
                        if let Ok(bytes) = std::fs::read(&p) {
                            combined.extend_from_slice(&bytes);
                            combined.extend_from_slice(b"\n\n");
                        }
                    }
                }
                ("monolai-all-services.log".to_string(), combined)
            } else {
                let svc = svc_opt.unwrap_or_else(|| "service".to_string());
                let file_name = format!("{}.log", sanitize_service_filename(&svc));
                let path = logs_dir.join(&file_name);
                let bytes = std::fs::read(&path).unwrap_or_default();
                (
                    format!("monolai-{}.log", sanitize_service_filename(&svc)),
                    bytes,
                )
            }
        })
        .await
        .unwrap_or_else(|_| ("monolai-logs.log".to_string(), Vec::new()))
    }

    /// Subscribe to live broadcast of log entries
    pub fn subscribe(&self) -> broadcast::Receiver<LogEntry> {
        self.broadcast_sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::init_db;

    #[tokio::test]
    async fn test_log_service_record_and_filtering() {
        let temp_dir =
            std::env::temp_dir().join(format!("monolai_test_logs_{}", uuid::Uuid::new_v4()));
        let logs_path = temp_dir.join("logs");
        let pool = init_db("sqlite::memory:").await.unwrap();
        let setting_repo = SettingRepository::new(pool).await;

        let service = LogService::new(logs_path.clone(), setting_repo);

        service
            .record("monolai", "INFO", "Server started on port 8080")
            .await;
        service
            .record("monolai", "WARN", "Config key missing")
            .await;
        service
            .record("llama.cpp: model-1", "INFO", "main: model loaded")
            .await;
        service
            .record("llama.cpp: model-1", "ERROR", "cuda out of memory")
            .await;

        let all_entries = service.get_entries(&LogFilterQuery::default()).await;
        assert_eq!(all_entries.len(), 4);

        // Filter by service
        let filter_monolai = LogFilterQuery {
            service: Some("monolai".to_string()),
            ..Default::default()
        };
        let monolai_entries = service.get_entries(&filter_monolai).await;
        assert_eq!(monolai_entries.len(), 2);

        // Filter by level
        let filter_error = LogFilterQuery {
            level: Some("ERROR".to_string()),
            ..Default::default()
        };
        let error_entries = service.get_entries(&filter_error).await;
        assert_eq!(error_entries.len(), 1);
        assert_eq!(error_entries[0].service, "llama.cpp: model-1");

        // Filter by search query
        let filter_search = LogFilterQuery {
            search: Some("memory".to_string()),
            ..Default::default()
        };
        let search_entries = service.get_entries(&filter_search).await;
        assert_eq!(search_entries.len(), 1);
        assert!(search_entries[0].message.contains("cuda out of memory"));

        // Verify services list
        let services = service.get_services().await;
        assert!(services.contains(&"all".to_string()));
        assert!(services.contains(&"monolai".to_string()));
        assert!(services.contains(&"llama.cpp: model-1".to_string()));

        // Verify clear
        service.clear_logs(Some("monolai")).await;
        let monolai_after_clear = service.get_entries(&filter_monolai).await;
        assert_eq!(monolai_after_clear.len(), 0);
        let remaining = service.get_entries(&LogFilterQuery::default()).await;
        assert_eq!(remaining.len(), 2);
    }

    #[tokio::test]
    async fn test_log_service_enable_log_flag() {
        let temp_dir =
            std::env::temp_dir().join(format!("monolai_test_logs_{}", uuid::Uuid::new_v4()));
        let logs_path = temp_dir.join("logs");
        let pool = init_db("sqlite::memory:").await.unwrap();
        let setting_repo = SettingRepository::new(pool).await;

        let service = LogService::new(logs_path.clone(), setting_repo.clone());

        // Default: enabled
        service
            .record("monolai", "INFO", "Should be recorded 1")
            .await;
        let entries = service.get_entries(&LogFilterQuery::default()).await;
        assert_eq!(entries.len(), 1);

        // Disable logging
        setting_repo.set("enable_log", "false").await.unwrap();
        service
            .record("monolai", "INFO", "Should NOT be recorded")
            .await;
        let entries_after = service.get_entries(&LogFilterQuery::default()).await;
        assert_eq!(entries_after.len(), 1);

        // Re-enable logging
        setting_repo.set("enable_log", "true").await.unwrap();
        service
            .record("monolai", "INFO", "Should be recorded 2")
            .await;
        let entries_re_enabled = service.get_entries(&LogFilterQuery::default()).await;
        assert_eq!(entries_re_enabled.len(), 2);
    }
}
