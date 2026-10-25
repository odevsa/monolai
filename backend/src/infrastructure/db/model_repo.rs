use crate::core::AppResult;
use crate::domain::{ModelPreset, ModelRecord};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ModelRepository {
    pool: SqlitePool,
}

pub fn normalize_flags(flags: &serde_json::Value) -> String {
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
        let mut models = sqlx::query_as::<_, ModelRecord>(
            "SELECT id, name, filename, relative_path, size_bytes, format, modality, architecture, source_repo, created_at FROM models ORDER BY name ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        for model in &mut models {
            model.presets = self.get_presets_for_model(&model.id).await?;
        }

        Ok(models)
    }

    pub async fn get_by_id(&self, id: &str) -> AppResult<Option<ModelRecord>> {
        let mut model = sqlx::query_as::<_, ModelRecord>(
            "SELECT id, name, filename, relative_path, size_bytes, format, modality, architecture, source_repo, created_at FROM models WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut m) = model {
            m.presets = self.get_presets_for_model(&m.id).await?;
        }

        Ok(model)
    }

    pub async fn find_by_relative_path(&self, relative_path: &str) -> AppResult<Option<ModelRecord>> {
        let mut model = sqlx::query_as::<_, ModelRecord>(
            "SELECT id, name, filename, relative_path, size_bytes, format, modality, architecture, source_repo, created_at FROM models WHERE relative_path = ?1",
        )
        .bind(relative_path)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut m) = model {
            m.presets = self.get_presets_for_model(&m.id).await?;
        }

        Ok(model)
    }

    pub async fn insert_model(&self, m: &ModelRecord) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO models (id, name, filename, relative_path, size_bytes, format, modality, architecture, source_repo) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(&m.id)
        .bind(&m.name)
        .bind(&m.filename)
        .bind(&m.relative_path)
        .bind(m.size_bytes)
        .bind(&m.format)
        .bind(&m.modality)
        .bind(&m.architecture)
        .bind(&m.source_repo)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn update_model(&self, id: &str, name: &str, modality: &str) -> AppResult<()> {
        sqlx::query("UPDATE models SET name = ?1, modality = ?2 WHERE id = ?3")
            .bind(name)
            .bind(modality)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_model_id(
        &self,
        old_id: &str,
        new_id: &str,
        name: &str,
        source_repo: Option<&str>,
    ) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE model_presets SET model_id = ?1 WHERE model_id = ?2")
            .bind(new_id)
            .bind(old_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE models SET id = ?1, name = ?2, source_repo = ?3 WHERE id = ?4")
            .bind(new_id)
            .bind(name)
            .bind(source_repo)
            .bind(old_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_model(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM models WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Preset Operations
    pub async fn get_presets_for_model(&self, model_id: &str) -> AppResult<Vec<ModelPreset>> {
        let presets = sqlx::query_as::<_, ModelPreset>(
            "SELECT id, model_id, name, runtime_id, flags, is_default, created_at FROM model_presets WHERE model_id = ?1 ORDER BY is_default DESC, name ASC",
        )
        .bind(model_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(presets)
    }

    pub async fn get_preset_by_id(&self, preset_id: &str) -> AppResult<Option<ModelPreset>> {
        let preset = sqlx::query_as::<_, ModelPreset>(
            "SELECT id, model_id, name, runtime_id, flags, is_default, created_at FROM model_presets WHERE id = ?1",
        )
        .bind(preset_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(preset)
    }

    pub async fn insert_preset(&self, p: &ModelPreset) -> AppResult<()> {
        if p.is_default {
            sqlx::query("UPDATE model_presets SET is_default = 0 WHERE model_id = ?1")
                .bind(&p.model_id)
                .execute(&self.pool)
                .await?;
        }

        sqlx::query(
            "INSERT INTO model_presets (id, model_id, name, runtime_id, flags, is_default) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(&p.id)
        .bind(&p.model_id)
        .bind(&p.name)
        .bind(&p.runtime_id)
        .bind(&p.flags)
        .bind(p.is_default)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_preset(&self, p: &ModelPreset) -> AppResult<()> {
        if p.is_default {
            sqlx::query("UPDATE model_presets SET is_default = 0 WHERE model_id = ?1 AND id != ?2")
                .bind(&p.model_id)
                .bind(&p.id)
                .execute(&self.pool)
                .await?;
        }

        sqlx::query(
            "UPDATE model_presets SET name = ?1, runtime_id = ?2, flags = ?3, is_default = ?4 WHERE id = ?5",
        )
        .bind(&p.name)
        .bind(&p.runtime_id)
        .bind(&p.flags)
        .bind(p.is_default)
        .bind(&p.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn set_default_preset(&self, model_id: &str, preset_id: &str) -> AppResult<()> {
        sqlx::query("UPDATE model_presets SET is_default = 0 WHERE model_id = ?1")
            .bind(model_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("UPDATE model_presets SET is_default = 1 WHERE id = ?1 AND model_id = ?2")
            .bind(preset_id)
            .bind(model_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_preset(&self, preset_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM model_presets WHERE id = ?1")
            .bind(preset_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
