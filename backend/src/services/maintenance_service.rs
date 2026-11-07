use crate::core::config::PathResolver;
use crate::core::error::{AppError, AppResult};
use crate::domain::MaintenanceResultDto;
use crate::services::ModelService;
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::Arc;

pub struct MaintenanceService {
    db: SqlitePool,
    model_service: Arc<ModelService>,
    path_resolver: Arc<PathResolver>,
}

impl MaintenanceService {
    pub fn new(
        db: SqlitePool,
        model_service: Arc<ModelService>,
        path_resolver: Arc<PathResolver>,
    ) -> Self {
        Self {
            db,
            model_service,
            path_resolver,
        }
    }

    pub async fn clean_database(&self) -> AppResult<MaintenanceResultDto> {
        tracing::info!(target: "monolai", "Starting in-process database wipe and migration reconstruction...");

        // 1. Unload all running models
        let _ = self.model_service.unload_all_models().await;

        // 2. Drop all views and tables
        sqlx::query("PRAGMA foreign_keys = OFF;")
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to disable foreign keys: {}", e)))?;

        let views: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='view' AND name NOT LIKE 'sqlite_%';",
        )
        .fetch_all(&self.db)
        .await
        .unwrap_or_default();

        for (view_name,) in views {
            let query = format!("DROP VIEW IF EXISTS \"{}\"", view_name);
            let _ = sqlx::query(&query).execute(&self.db).await;
        }

        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';",
        )
        .fetch_all(&self.db)
        .await
        .unwrap_or_default();

        let mut dropped_tables_count = 0;
        for (table_name,) in tables {
            let query = format!("DROP TABLE IF EXISTS \"{}\"", table_name);
            if sqlx::query(&query).execute(&self.db).await.is_ok() {
                dropped_tables_count += 1;
            }
        }

        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to re-enable foreign keys: {}", e)))?;

        // 3. Rebuild migrations from scratch
        let migrator = sqlx::migrate!("./migrations");
        migrator
            .run(&self.db)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to rebuild migrations: {}", e)))?;

        // 4. Remove processes.json
        let mut files_removed = 0;
        let db_dir = self
            .path_resolver
            .db_path
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let proc_file = db_dir.join("processes.json");
        if proc_file.exists() && std::fs::remove_file(&proc_file).is_ok() {
            files_removed += 1;
        }

        tracing::info!(
            target: "monolai",
            "Database wiped and migrations rebuilt successfully (dropped {} tables).",
            dropped_tables_count
        );

        Ok(MaintenanceResultDto {
            success: true,
            message: format!(
                "Database reset and migrations rebuilt successfully ({} tables dropped).",
                dropped_tables_count
            ),
            files_removed,
        })
    }

    pub async fn clean_storage(&self) -> AppResult<MaintenanceResultDto> {
        tracing::info!(target: "monolai", "Starting storage purge (models and runtimes directories)...");

        // 1. Unload all running models
        let _ = self.model_service.unload_all_models().await;

        // 2. Empty models directory
        let models_removed = empty_directory(&self.path_resolver.models_dir)
            .map_err(|e| AppError::Internal(format!("Failed to clean models directory: {}", e)))?;

        // 3. Empty runtimes directory
        let runtimes_removed = empty_directory(&self.path_resolver.runtimes_dir)
            .map_err(|e| AppError::Internal(format!("Failed to clean runtimes directory: {}", e)))?;

        // 4. Clear models table from DB so state matches disk
        let _ = sqlx::query("DELETE FROM models").execute(&self.db).await;
        let _ = sqlx::query("DELETE FROM model_presets").execute(&self.db).await;
        let _ = sqlx::query("DELETE FROM downloads").execute(&self.db).await;

        let total_removed = models_removed + runtimes_removed;

        tracing::info!(
            target: "monolai",
            "Storage purge completed successfully ({} items removed).",
            total_removed
        );

        Ok(MaintenanceResultDto {
            success: true,
            message: format!(
                "Storage purged successfully ({} models/runtimes items removed).",
                total_removed
            ),
            files_removed: total_removed,
        })
    }

    pub async fn factory_reset(&self) -> AppResult<MaintenanceResultDto> {
        tracing::info!(target: "monolai", "Starting full factory reset...");

        // 1. Clean storage
        let storage_res = self.clean_storage().await?;

        // 2. Clean database
        let db_res = self.clean_database().await?;

        // 3. Delete config.yaml
        let mut config_removed = 0;
        let config_path = &self.path_resolver.config_path;
        if config_path.exists() && std::fs::remove_file(config_path).is_ok() {
            config_removed += 1;
        }

        let fallback_cfg = Path::new("config.yaml");
        if fallback_cfg.exists() && std::fs::remove_file(fallback_cfg).is_ok() {
            config_removed += 1;
        }

        let total_removed = storage_res.files_removed + db_res.files_removed + config_removed;

        tracing::info!(
            target: "monolai",
            "Factory reset completed successfully ({} items/files removed).",
            total_removed
        );

        Ok(MaintenanceResultDto {
            success: true,
            message: "Factory reset completed successfully. Database, storage, and configurations have been reset.".to_string(),
            files_removed: total_removed,
        })
    }
}

pub fn empty_directory(dir: &Path) -> std::io::Result<usize> {
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
        return Ok(0);
    }

    let mut count = 0;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
            count += 1;
        } else if path.is_file() {
            std::fs::remove_file(&path)?;
            count += 1;
        }
    }

    // Ensure the directory itself exists after emptying
    let _ = std::fs::create_dir_all(dir);

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_directory() {
        let dir = std::env::temp_dir().join(format!("test_clean_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&dir);
        let path = &dir;

        let sub_file = path.join("test.txt");
        std::fs::write(&sub_file, "hello").unwrap();

        let sub_folder = path.join("subdir");
        std::fs::create_dir(&sub_folder).unwrap();
        std::fs::write(sub_folder.join("nested.bin"), "data").unwrap();

        let removed = empty_directory(path).expect("empty_directory");
        assert_eq!(removed, 2);
        assert!(path.exists());
        assert_eq!(std::fs::read_dir(path).unwrap().count(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_clean_database_in_memory() {
        let pool = crate::infrastructure::db::init_db(":memory:")
            .await
            .expect("init_db");

        // Verify tables exist from migrations
        let initial_tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(!initial_tables.is_empty());

        // Insert a dummy chat
        let _ = sqlx::query(
            "INSERT INTO chats (id, title, created_at, updated_at) VALUES ('c1', 'Test', 'now', 'now')",
        )
        .execute(&pool)
        .await;

        let chats_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chats")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(chats_count.0, 1);

        // Manually drop and re-run migrations as in clean_database
        sqlx::query("PRAGMA foreign_keys = OFF;")
            .execute(&pool)
            .await
            .unwrap();
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        for (table_name,) in tables {
            let query = format!("DROP TABLE IF EXISTS \"{}\"", table_name);
            let _ = sqlx::query(&query).execute(&pool).await;
        }
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await
            .unwrap();

        let migrator = sqlx::migrate!("./migrations");
        migrator.run(&pool).await.unwrap();

        // Chats table exists again and is clean
        let new_chats_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chats")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(new_chats_count.0, 0);
    }
}
