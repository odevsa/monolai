use crate::core::error::AppResult;
use crate::domain::{AudioGenerationRecord, CreateAudioGenerationPayload};
use crate::infrastructure::db::AudioRepository;

#[derive(Clone)]
pub struct AudioService {
    repo: AudioRepository,
}

impl AudioService {
    pub fn new(repo: AudioRepository) -> Self {
        Self { repo }
    }

    pub async fn list_audio(
        &self,
        audio_type: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<Vec<AudioGenerationRecord>> {
        let limit = limit.unwrap_or(100).max(1).min(500);
        let offset = offset.unwrap_or(0).max(0);
        self.repo.get_all(audio_type, limit, offset).await
    }

    pub async fn get_audio_by_id(&self, id: &str) -> AppResult<Option<AudioGenerationRecord>> {
        self.repo.get_by_id(id).await
    }

    pub async fn create_audio(&self, payload: CreateAudioGenerationPayload) -> AppResult<AudioGenerationRecord> {
        self.repo.insert(&payload).await
    }

    pub async fn delete_audio(&self, id: &str) -> AppResult<bool> {
        self.repo.delete(id).await
    }

    pub async fn clear_audio(&self) -> AppResult<u64> {
        self.repo.delete_all().await
    }
}

