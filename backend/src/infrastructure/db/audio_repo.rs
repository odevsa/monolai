use crate::core::AppResult;
use crate::domain::{AudioGenerationRecord, CreateAudioGenerationPayload};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AudioRepository {
    pool: SqlitePool,
}

impl AudioRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_all(
        &self,
        audio_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<AudioGenerationRecord>> {
        let rows = if let Some(atype) = audio_type {
            sqlx::query_as::<_, AudioGenerationRecord>(
                "SELECT id, audio_type, model, prompt, audio_url, voice, speed, format, duration_secs, transcript, filename, language, word_count, created_at \
                 FROM audios \
                 WHERE audio_type = ? \
                 ORDER BY created_at DESC \
                 LIMIT ? OFFSET ?"
            )
            .bind(atype)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, AudioGenerationRecord>(
                "SELECT id, audio_type, model, prompt, audio_url, voice, speed, format, duration_secs, transcript, filename, language, word_count, created_at \
                 FROM audios \
                 ORDER BY created_at DESC \
                 LIMIT ? OFFSET ?"
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows)
    }

    pub async fn get_by_id(&self, id: &str) -> AppResult<Option<AudioGenerationRecord>> {
        let row = sqlx::query_as::<_, AudioGenerationRecord>(
            "SELECT id, audio_type, model, prompt, audio_url, voice, speed, format, duration_secs, transcript, filename, language, word_count, created_at \
             FROM audios \
             WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn insert(&self, payload: &CreateAudioGenerationPayload) -> AppResult<AudioGenerationRecord> {
        let id = payload
            .id
            .clone()
            .unwrap_or_else(|| format!("audio-{}", uuid::Uuid::new_v4()));

        sqlx::query(
            "INSERT INTO audios (id, audio_type, model, prompt, audio_url, voice, speed, format, duration_secs, transcript, filename, language, word_count, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(&id)
        .bind(&payload.audio_type)
        .bind(&payload.model)
        .bind(&payload.prompt)
        .bind(&payload.audio_url)
        .bind(&payload.voice)
        .bind(payload.speed)
        .bind(&payload.format)
        .bind(payload.duration_secs)
        .bind(&payload.transcript)
        .bind(&payload.filename)
        .bind(&payload.language)
        .bind(payload.word_count)
        .execute(&self.pool)
        .await?;

        self.get_by_id(&id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound.into())
    }

    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        let res = sqlx::query("DELETE FROM audios WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() > 0)
    }

    pub async fn delete_all(&self) -> AppResult<u64> {
        let res = sqlx::query("DELETE FROM audios")
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
    async fn test_audio_repository_crud() {
        let pool = create_test_pool().await;
        let repo = AudioRepository::new(pool);

        let tts_payload = CreateAudioGenerationPayload {
            id: Some("audio-test-tts".to_string()),
            audio_type: "tts".to_string(),
            model: "tts-model".to_string(),
            prompt: Some("Hello world".to_string()),
            audio_url: Some("data:audio/wav;base64,sample".to_string()),
            voice: Some("en-US-Jenny".to_string()),
            speed: Some(1.0),
            format: Some("wav".to_string()),
            duration_secs: Some(1.5),
            transcript: None,
            filename: None,
            language: None,
            word_count: None,
        };

        let created_tts = repo.insert(&tts_payload).await.unwrap();
        assert_eq!(created_tts.id, "audio-test-tts");
        assert_eq!(created_tts.audio_type, "tts");

        let asr_payload = CreateAudioGenerationPayload {
            id: Some("audio-test-asr".to_string()),
            audio_type: "asr".to_string(),
            model: "whisper-base".to_string(),
            prompt: None,
            audio_url: None,
            voice: None,
            speed: None,
            format: None,
            duration_secs: None,
            transcript: Some("Transcribed speech text".to_string()),
            filename: Some("voice.wav".to_string()),
            language: Some("en".to_string()),
            word_count: Some(3),
        };

        let created_asr = repo.insert(&asr_payload).await.unwrap();
        assert_eq!(created_asr.id, "audio-test-asr");

        let all = repo.get_all(None, 50, 0).await.unwrap();
        assert_eq!(all.len(), 2);

        let tts_only = repo.get_all(Some("tts"), 50, 0).await.unwrap();
        assert_eq!(tts_only.len(), 1);
        assert_eq!(tts_only[0].id, "audio-test-tts");

        let asr_only = repo.get_all(Some("asr"), 50, 0).await.unwrap();
        assert_eq!(asr_only.len(), 1);
        assert_eq!(asr_only[0].id, "audio-test-asr");

        let deleted = repo.delete("audio-test-tts").await.unwrap();
        assert!(deleted);

        let all_after = repo.get_all(None, 50, 0).await.unwrap();
        assert_eq!(all_after.len(), 1);

        repo.delete_all().await.unwrap();
        let empty = repo.get_all(None, 50, 0).await.unwrap();
        assert_eq!(empty.len(), 0);
    }
}

