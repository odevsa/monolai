use crate::core::AppResult;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SettingRepository {
    pool: SqlitePool,
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl SettingRepository {
    pub async fn new(pool: SqlitePool) -> Self {
        let repo = Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
        };
        let _ = repo.refresh_cache().await;
        repo
    }

    pub async fn refresh_cache(&self) -> AppResult<()> {
        let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM settings")
            .fetch_all(&self.pool)
            .await?;

        let mut map = self.cache.write().await;
        map.clear();
        for (k, v) in rows {
            map.insert(k, v);
        }
        Ok(())
    }

    pub async fn get_all(&self) -> AppResult<HashMap<String, String>> {
        let map = self.cache.read().await;
        if !map.is_empty() {
            return Ok(map.clone());
        }
        drop(map);

        self.refresh_cache().await?;
        let map = self.cache.read().await;
        Ok(map.clone())
    }

    pub async fn get(&self, key: &str) -> AppResult<Option<String>> {
        {
            let map = self.cache.read().await;
            if let Some(val) = map.get(key) {
                return Ok(Some(val.clone()));
            }
        }

        let row = sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(ref val) = row {
            let mut map = self.cache.write().await;
            map.insert(key.to_string(), val.clone());
        }

        Ok(row)
    }

    pub async fn get_parsed<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
        self.get(key)
            .await
            .ok()
            .flatten()
            .and_then(|s| s.parse::<T>().ok())
            .unwrap_or(default)
    }

    pub async fn set(&self, key: &str, value: &str) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;

        let mut map = self.cache.write().await;
        map.insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM settings WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;

        let mut map = self.cache.write().await;
        map.remove(key);
        Ok(())
    }
}
