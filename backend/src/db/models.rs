use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct ModelRecord {
    pub id: String,
    pub runtime: String,
    pub flags: String,
    pub created_at: String,
    #[sqlx(default)]
    pub file_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateModelPayload {
    pub id: String,
    pub runtime: String,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub flags: serde_json::Value,
}

/// Get all registered models from the database.
pub async fn get_all_models(pool: &SqlitePool) -> Result<Vec<ModelRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ModelRecord>(
        "SELECT id, runtime, flags, created_at FROM models ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get a single registered model by ID.
pub async fn get_model_by_id(pool: &SqlitePool, id: &str) -> Result<Option<ModelRecord>, sqlx::Error> {
    let row = sqlx::query_as::<_, ModelRecord>(
        "SELECT id, runtime, flags, created_at FROM models WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Insert a new registered model into the database.
pub async fn insert_model(pool: &SqlitePool, payload: &CreateModelPayload) -> Result<ModelRecord, sqlx::Error> {
    let flags_str = payload.flags.to_string();

    sqlx::query(
        "INSERT INTO models (id, runtime, flags) VALUES (?1, ?2, ?3)",
    )
    .bind(&payload.id)
    .bind(&payload.runtime)
    .bind(&flags_str)
    .execute(pool)
    .await?;

    let model = get_model_by_id(pool, &payload.id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

    Ok(model)
}

/// Update an existing model by ID.
pub async fn update_model(
    pool: &SqlitePool,
    id: &str,
    payload: &CreateModelPayload,
) -> Result<ModelRecord, sqlx::Error> {
    let flags_str = payload.flags.to_string();

    sqlx::query(
        "UPDATE models SET runtime = ?1, flags = ?2 WHERE id = ?3",
    )
    .bind(&payload.runtime)
    .bind(&flags_str)
    .bind(id)
    .execute(pool)
    .await?;

    let model = get_model_by_id(pool, id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

    Ok(model)
}

/// Delete a model by ID.
pub async fn delete_model(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM models WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
