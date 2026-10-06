use crate::core::AppResult;
use crate::domain::{CreateModelPayload, ModelRecord};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ModelRepository {
    pool: SqlitePool,
}

impl ModelRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> AppResult<Vec<ModelRecord>> {
        let rows = sqlx::query_as::<_, ModelRecord>(
            "SELECT id, runtime, flags, created_at FROM models ORDER BY id ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_by_id(&self, id: &str) -> AppResult<Option<ModelRecord>> {
        let row = sqlx::query_as::<_, ModelRecord>(
            "SELECT id, runtime, flags, created_at FROM models WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn insert(&self, payload: &CreateModelPayload) -> AppResult<ModelRecord> {
        let flags_str = payload.flags.to_string();

        sqlx::query("INSERT INTO models (id, runtime, flags) VALUES (?1, ?2, ?3)")
            .bind(&payload.id)
            .bind(&payload.runtime)
            .bind(&flags_str)
            .execute(&self.pool)
            .await?;

        let model = self
            .get_by_id(&payload.id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        Ok(model)
    }

    pub async fn update(&self, id: &str, payload: &CreateModelPayload) -> AppResult<ModelRecord> {
        let flags_str = payload.flags.to_string();

        sqlx::query("UPDATE models SET runtime = ?1, flags = ?2 WHERE id = ?3")
            .bind(&payload.runtime)
            .bind(&flags_str)
            .bind(id)
            .execute(&self.pool)
            .await?;

        let model = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        Ok(model)
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM models WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
