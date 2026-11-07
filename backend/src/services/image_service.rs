use crate::core::error::AppResult;
use crate::domain::{CreateGeneratedImagePayload, GeneratedImageRecord};
use crate::infrastructure::db::ImageRepository;

#[derive(Clone)]
pub struct ImageService {
    repo: ImageRepository,
}

impl ImageService {
    pub fn new(repo: ImageRepository) -> Self {
        Self { repo }
    }

    pub async fn list_images(&self, limit: Option<i64>, offset: Option<i64>) -> AppResult<Vec<GeneratedImageRecord>> {
        let limit = limit.unwrap_or(100).max(1).min(500);
        let offset = offset.unwrap_or(0).max(0);
        self.repo.get_all(limit, offset).await
    }

    pub async fn get_image_by_id(&self, id: &str) -> AppResult<Option<GeneratedImageRecord>> {
        self.repo.get_by_id(id).await
    }

    pub async fn create_image(&self, payload: CreateGeneratedImagePayload) -> AppResult<GeneratedImageRecord> {
        self.repo.insert(&payload).await
    }

    pub async fn delete_image(&self, id: &str) -> AppResult<bool> {
        self.repo.delete(id).await
    }

    pub async fn clear_images(&self) -> AppResult<u64> {
        self.repo.delete_all().await
    }
}

