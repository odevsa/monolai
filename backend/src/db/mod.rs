pub mod chats;
pub mod models;
pub mod settings;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;

/// Initialize SQLite pool and execute embedded migrations.
pub async fn init_db(database_url: &str) -> Result<SqlitePool, Box<dyn std::error::Error + Send + Sync>> {
    let clean_path = database_url
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:");

    if !clean_path.is_empty() && clean_path != ":memory:" {
        let path = Path::new(clean_path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
    }

    let connection_url = if database_url.starts_with("sqlite:") {
        database_url.to_string()
    } else if database_url.starts_with('/') {
        format!("sqlite://{}?mode=rwc", database_url)
    } else {
        format!("sqlite:{}?mode=rwc", database_url)
    };

    let options = SqliteConnectOptions::from_str(&connection_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
