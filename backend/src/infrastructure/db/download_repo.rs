use crate::core::AppResult;
use crate::domain::DownloadRecord;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct DownloadRepository {
    pool: SqlitePool,
}

impl DownloadRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> AppResult<Vec<DownloadRecord>> {
        let rows = sqlx::query_as::<_, DownloadRecord>(
            "SELECT id, model_name, url, file_path, total_bytes, downloaded_bytes, status, error_message, created_at, updated_at FROM downloads ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_by_id(&self, id: &str) -> AppResult<Option<DownloadRecord>> {
        let row = sqlx::query_as::<_, DownloadRecord>(
            "SELECT id, model_name, url, file_path, total_bytes, downloaded_bytes, status, error_message, created_at, updated_at FROM downloads WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn insert(&self, d: &DownloadRecord) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO downloads (id, model_name, url, file_path, total_bytes, downloaded_bytes, status, error_message) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&d.id)
        .bind(&d.model_name)
        .bind(&d.url)
        .bind(&d.file_path)
        .bind(d.total_bytes)
        .bind(d.downloaded_bytes)
        .bind(&d.status)
        .bind(&d.error_message)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_progress(
        &self,
        id: &str,
        downloaded_bytes: i64,
        total_bytes: i64,
    ) -> AppResult<()> {
        sqlx::query(
            "UPDATE downloads SET downloaded_bytes = ?1, total_bytes = ?2, updated_at = datetime('now') WHERE id = ?3",
        )
        .bind(downloaded_bytes)
        .bind(total_bytes)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            "UPDATE downloads SET status = ?1, error_message = ?2, updated_at = datetime('now') WHERE id = ?3",
        )
        .bind(status)
        .bind(error_message)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM downloads WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_inactive(&self) -> AppResult<u64> {
        let res =
            sqlx::query("DELETE FROM downloads WHERE status NOT IN ('downloading', 'queued')")
                .execute(&self.pool)
                .await?;
        Ok(res.rows_affected())
    }
}
