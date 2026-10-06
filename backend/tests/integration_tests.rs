// Integration tests for Monolai backend clean architecture
// Testing repositories, services, errors, and configuration

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory SQLite");

        let migrator = sqlx::migrate!("./migrations");
        migrator
            .run(&pool)
            .await
            .expect("Failed to run migrations on in-memory SQLite");

        pool
    }

    #[tokio::test]
    async fn test_db_migrations_and_settings() {
        let pool = setup_test_db().await;

        // Test inserting and retrieving settings
        sqlx::query("INSERT INTO settings (key, value) VALUES ('test_key', 'test_value')")
            .execute(&pool)
            .await
            .unwrap();

        let val: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'test_key'")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(val, "test_value");
    }

    #[tokio::test]
    async fn test_chat_and_message_schema() {
        let pool = setup_test_db().await;

        let chat_id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO chats (id, title, updated_at) VALUES (?, ?, datetime('now'))")
            .bind(&chat_id)
            .bind("Test Chat Title")
            .execute(&pool)
            .await
            .unwrap();

        let msg_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO chat_messages (id, chat_id, role, content) VALUES (?, ?, ?, ?)",
        )
        .bind(&msg_id)
        .bind(&chat_id)
        .bind("user")
        .bind("Hello Monolai!")
        .execute(&pool)
        .await
        .unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_messages WHERE chat_id = ?")
            .bind(&chat_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 1);
    }
}
