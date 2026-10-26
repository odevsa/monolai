use crate::core::{AppError, AppResult};
use crate::domain::RepositoryRecord;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct RepositoryRepository {
    pool: SqlitePool,
}

impl RepositoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> AppResult<Vec<RepositoryRecord>> {
        let rows = sqlx::query_as::<_, RepositoryRecord>(
            "SELECT id, name, repo_type, base_url, api_token, enabled, is_builtin, config_json, created_at FROM repositories ORDER BY is_builtin DESC, name ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_enabled(&self) -> AppResult<Vec<RepositoryRecord>> {
        let rows = sqlx::query_as::<_, RepositoryRecord>(
            "SELECT id, name, repo_type, base_url, api_token, enabled, is_builtin, config_json, created_at FROM repositories WHERE enabled = 1 ORDER BY is_builtin DESC, name ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_by_id(&self, id: &str) -> AppResult<Option<RepositoryRecord>> {
        let row = sqlx::query_as::<_, RepositoryRecord>(
            "SELECT id, name, repo_type, base_url, api_token, enabled, is_builtin, config_json, created_at FROM repositories WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn insert(&self, r: &RepositoryRecord) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO repositories (id, name, repo_type, base_url, api_token, enabled, is_builtin, config_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&r.id)
        .bind(&r.name)
        .bind(&r.repo_type)
        .bind(&r.base_url)
        .bind(&r.api_token)
        .bind(r.enabled)
        .bind(r.is_builtin)
        .bind(&r.config_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update(&self, r: &RepositoryRecord) -> AppResult<()> {
        sqlx::query(
            "UPDATE repositories SET name = ?1, base_url = ?2, api_token = ?3, enabled = ?4, config_json = ?5 WHERE id = ?6",
        )
        .bind(&r.name)
        .bind(&r.base_url)
        .bind(&r.api_token)
        .bind(r.enabled)
        .bind(&r.config_json)
        .bind(&r.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        let existing = self.get_by_id(id).await?;
        if let Some(r) = existing {
            if r.is_builtin {
                return Err(AppError::bad_request(
                    "Built-in repositories cannot be deleted. You can disable them instead.",
                ));
            }
        }
        sqlx::query("DELETE FROM repositories WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
