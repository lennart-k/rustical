#[cfg(test)]
mod tests {
    use crate::{
        principal_store::SqlitePrincipalStore,
        tests::{TestStoreContext, test_store_context},
    };
    use rstest::rstest;
    use rustical_store::auth::AuthenticationProvider;
    use sqlx::{SqlitePool, migrate::Migrator};

    async fn last_used(store: &SqlitePrincipalStore) -> Option<chrono::DateTime<chrono::Utc>> {
        let tokens = store.get_app_tokens("user").await.unwrap();
        assert_eq!(tokens.len(), 1);
        tokens[0].last_used
    }

    #[rstest]
    #[tokio::test]
    async fn test_app_token_last_used(
        #[from(test_store_context)]
        #[future]
        context: TestStoreContext,
    ) {
        let context = context.await;
        let store = context.principal_store;
        assert_eq!(last_used(&store).await, None, "a new token was never used");

        assert!(
            store
                .validate_app_token("user", "wrong")
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            last_used(&store).await,
            None,
            "a wrong token records nothing"
        );

        assert!(
            store
                .validate_app_token("user", "pass")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            last_used(&store).await.is_some(),
            "a valid token records its use"
        );

        // Within the hour a second use is not written again, also not through a clone.
        sqlx::query("UPDATE app_tokens SET last_used = '2000-01-01 00:00:00'")
            .execute(&context.db)
            .await
            .unwrap();
        assert!(
            store
                .clone()
                .validate_app_token("user", "pass")
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            last_used(&store).await.unwrap().to_rfc3339(),
            "2000-01-01T00:00:00+00:00"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_app_token_removed_and_touched(
        #[from(test_store_context)]
        #[future]
        context: TestStoreContext,
    ) {
        let store = context.await.principal_store;
        let id = store.get_app_tokens("user").await.unwrap()[0].id.clone();
        store.remove_app_token("user", &id).await.unwrap();
        // Touching a token that no longer exists is harmless.
        store.touch_app_token("user", &id).await.unwrap();
        assert!(store.get_app_tokens("user").await.unwrap().is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn test_app_token_valid_when_recording_its_use_fails(
        #[from(test_store_context)]
        #[future]
        context: TestStoreContext,
    ) {
        let context = context.await;
        sqlx::query(
            "CREATE TRIGGER no_touch BEFORE UPDATE OF last_used ON app_tokens BEGIN SELECT RAISE(ABORT, 'read-only'); END",
        )
        .execute(&context.db)
        .await
        .unwrap();
        let store = context.principal_store;
        assert!(
            store
                .validate_app_token("user", "pass")
                .await
                .unwrap()
                .is_some(),
            "a failed write must not reject a valid token"
        );
        assert_eq!(last_used(&store).await, None);
    }

    #[tokio::test]
    async fn test_app_token_last_used_migration_keeps_tokens() {
        // A database migrated by the previous release, with a token in it.
        let all = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        let previous = std::env::temp_dir().join(format!(
            "rustical-migrations-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&previous).unwrap();
        for entry in std::fs::read_dir(&all).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name();
            if !name.to_string_lossy().contains("app_token_last_used") {
                std::fs::copy(entry.path(), previous.join(name)).unwrap();
            }
        }
        let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
        Migrator::new(previous.as_path())
            .await
            .unwrap()
            .run(&db)
            .await
            .unwrap();
        std::fs::remove_dir_all(&previous).unwrap();
        sqlx::query("INSERT INTO principals (id, principal_type) VALUES ('user', 'individual')")
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO app_tokens (id, principal, token, displayname) VALUES ('t', 'user', 'x', 'old')")
            .execute(&db)
            .await
            .unwrap();

        sqlx::migrate!("./migrations").run(&db).await.unwrap();

        let store = SqlitePrincipalStore::new(db);
        let tokens = store.get_app_tokens("user").await.unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].name, "old");
        assert_eq!(tokens[0].last_used, None);
    }
}
