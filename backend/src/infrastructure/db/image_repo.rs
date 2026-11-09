use crate::core::AppResult;
use crate::domain::{CreateGeneratedImagePayload, GeneratedImageRecord};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ImageRepository {
    pool: SqlitePool,
}

impl ImageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self, limit: i64, offset: i64) -> AppResult<Vec<GeneratedImageRecord>> {
        let rows = sqlx::query_as::<_, GeneratedImageRecord>(
            "SELECT id, prompt, negative_prompt, model, size, steps, cfg_scale, seed, src, generation_time_secs, created_at \
             FROM images \
             ORDER BY created_at DESC \
             LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_by_id(&self, id: &str) -> AppResult<Option<GeneratedImageRecord>> {
        let row = sqlx::query_as::<_, GeneratedImageRecord>(
            "SELECT id, prompt, negative_prompt, model, size, steps, cfg_scale, seed, src, generation_time_secs, created_at \
             FROM images \
             WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn insert(&self, payload: &CreateGeneratedImagePayload) -> AppResult<GeneratedImageRecord> {
        let id = payload
            .id
            .clone()
            .unwrap_or_else(|| format!("img-{}", uuid::Uuid::new_v4()));

        sqlx::query(
            "INSERT INTO images (id, prompt, negative_prompt, model, size, steps, cfg_scale, seed, src, generation_time_secs, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(&id)
        .bind(&payload.prompt)
        .bind(&payload.negative_prompt)
        .bind(&payload.model)
        .bind(&payload.size)
        .bind(payload.steps)
        .bind(payload.cfg_scale)
        .bind(payload.seed)
        .bind(&payload.src)
        .bind(payload.generation_time_secs)
        .execute(&self.pool)
        .await?;

        self.get_by_id(&id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound.into())
    }

    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        let res = sqlx::query("DELETE FROM images WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() > 0)
    }

    pub async fn delete_all(&self) -> AppResult<u64> {
        let res = sqlx::query("DELETE FROM images")
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_image_repository_crud() {
        let pool = create_test_pool().await;
        let repo = ImageRepository::new(pool);

        let payload = CreateGeneratedImagePayload {
            id: Some("test-img-1".to_string()),
            prompt: "A beautiful sunset".to_string(),
            negative_prompt: Some("blurry".to_string()),
            model: "sd-1.5".to_string(),
            size: "512x512".to_string(),
            steps: 20,
            cfg_scale: 7.5,
            seed: Some(42),
            src: "data:image/png;base64,sample".to_string(),
            generation_time_secs: Some(3.2),
        };

        let created = repo.insert(&payload).await.unwrap();
        assert_eq!(created.id, "test-img-1");
        assert_eq!(created.prompt, "A beautiful sunset");

        let fetched = repo.get_by_id("test-img-1").await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().model, "sd-1.5");

        let all = repo.get_all(50, 0).await.unwrap();
        assert_eq!(all.len(), 1);

        let deleted = repo.delete("test-img-1").await.unwrap();
        assert!(deleted);

        let all_after = repo.get_all(50, 0).await.unwrap();
        assert_eq!(all_after.len(), 0);

        // Test inserting large image payload (e.g. 4MB base64 string for 1024x1024)
        let large_src = format!("data:image/png;base64,{}", "A".repeat(4 * 1024 * 1024));
        let large_payload = CreateGeneratedImagePayload {
            id: Some("test-large-1024".to_string()),
            prompt: "1024x1024 high-res photo".to_string(),
            negative_prompt: None,
            model: "flux-schnell".to_string(),
            size: "1024x1024".to_string(),
            steps: 30,
            cfg_scale: 5.0,
            seed: Some(12345),
            src: large_src.clone(),
            generation_time_secs: Some(8.5),
        };

        let created_large = repo.insert(&large_payload).await.unwrap();
        assert_eq!(created_large.id, "test-large-1024");
        assert_eq!(created_large.src.len(), large_src.len());

        let fetched_large = repo.get_by_id("test-large-1024").await.unwrap().expect("should find large image");
        assert_eq!(fetched_large.size, "1024x1024");
        assert_eq!(fetched_large.src.len(), large_src.len());
    }
}

