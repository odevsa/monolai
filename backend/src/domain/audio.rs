use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioGenerationRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub audio_type: String,
    pub model: String,
    pub prompt: Option<String>,
    pub audio_url: Option<String>,
    pub voice: Option<String>,
    pub speed: Option<f64>,
    pub format: Option<String>,
    pub duration_secs: Option<f64>,
    pub transcript: Option<String>,
    pub filename: Option<String>,
    pub language: Option<String>,
    pub word_count: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAudioGenerationPayload {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub audio_type: String,
    pub model: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub audio_url: Option<String>,
    #[serde(default)]
    pub voice: Option<String>,
    #[serde(default)]
    pub speed: Option<f64>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub duration_secs: Option<f64>,
    #[serde(default)]
    pub transcript: Option<String>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub word_count: Option<i64>,
}

