use crate::core::AppResult;
use crate::domain::{CreateModelPayload, ModelRecord};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ModelRepository {
    pool: SqlitePool,
}

fn normalize_flags(flags: &serde_json::Value) -> String {
    let mut val = flags.clone();
    while let serde_json::Value::String(s) = &val {
        if let Ok(inner) = serde_json::from_str(s) {
            val = inner;
        } else {
            break;
        }
    }
    val.to_string()
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
        let flags_str = normalize_flags(&payload.flags);

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
        let flags_str = normalize_flags(&payload.flags);

        sqlx::query("UPDATE models SET id = ?1, runtime = ?2, flags = ?3 WHERE id = ?4")
            .bind(&payload.id)
            .bind(&payload.runtime)
            .bind(&flags_str)
            .bind(id)
            .execute(&self.pool)
            .await?;

        let model = self
            .get_by_id(&payload.id)
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
