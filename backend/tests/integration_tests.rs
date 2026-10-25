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
        sqlx::query("INSERT INTO chat_messages (id, chat_id, role, content) VALUES (?, ?, ?, ?)")
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

    #[tokio::test]
    async fn test_default_builtin_repositories() {
        let pool = setup_test_db().await;

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM repositories WHERE is_builtin = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_models_and_presets_schema() {
        let pool = setup_test_db().await;

        let model_id = "test-llama-3";
        sqlx::query("INSERT INTO models (id, name, filename, relative_path, size_bytes, format, modality) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(model_id)
            .bind("Test LLaMA 3")
            .bind("test.gguf")
            .bind("text-to-text/test.gguf")
            .bind(4000000000i64)
            .bind("gguf")
            .bind("text-to-text")
            .execute(&pool)
            .await
            .unwrap();

        let preset_id = "test-llama-3:default";
        sqlx::query("INSERT INTO model_presets (id, model_id, name, runtime_id, flags, is_default) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(preset_id)
            .bind(model_id)
            .bind("Default")
            .bind("llama-cpp")
            .bind("{\"--model\": \"test.gguf\"}")
            .bind(1)
            .execute(&pool)
            .await
            .unwrap();

        let preset_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM model_presets WHERE model_id = ?")
            .bind(model_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(preset_count, 1);

        // Test cascade deletion
        sqlx::query("DELETE FROM models WHERE id = ?")
            .bind(model_id)
            .execute(&pool)
            .await
            .unwrap();

        let after_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM model_presets WHERE model_id = ?")
            .bind(model_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(after_count, 0);
    }
}
