#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
pub use error::Error;
use serde::Serialize;
use sqlx::{Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};
use tracing::info;
pub mod addressbook_store;
pub mod calendar_store;
pub mod error;
pub mod principal_store;
pub mod subscription_store;

// Begin statement for write transactions
pub const BEGIN_IMMEDIATE: &str = "BEGIN IMMEDIATE";

#[cfg(any(test, feature = "test"))]
pub mod tests;

#[derive(Debug, Clone, Serialize, sqlx::Type)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ChangeOperation {
    // There's no distinction between Add and Modify
    Add,
    Delete,
}

#[derive(Debug, Clone)]
pub struct SqliteStore {
    db: SqlitePool,
}

impl SqliteStore {
    #[must_use]
    pub const fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Read the persisted DAV Push VAPID keypair PEM, if one has been generated.
    /// The single `server_settings` row is seeded by the migration; the column is
    /// NULL until a key is stored.
    pub async fn get_vapid_key(&self) -> Result<Option<String>, rustical_store::Error> {
        Ok(
            sqlx::query_scalar!("SELECT vapid_key FROM server_settings WHERE id = 1")
                .fetch_one(&self.db)
                .await
                .map_err(crate::Error::from)?,
        )
    }

    /// Persist the DAV Push VAPID keypair PEM, but only if one isn't already set.
    /// The key is generate-once and never rotated here, so a conditional update
    /// makes first-run creation race-safe (a concurrent process can't clobber a
    /// key that's already in use).
    pub async fn set_vapid_key_if_unset(&self, pem: &str) -> Result<(), rustical_store::Error> {
        sqlx::query!(
            "UPDATE server_settings SET vapid_key = ? WHERE id = 1 AND vapid_key IS NULL",
            pem
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }
}

pub async fn create_db_pool(db_url: &str, migrate: bool) -> Result<Pool<Sqlite>, sqlx::Error> {
    let options: SqliteConnectOptions = db_url.parse()?;

    let db = SqlitePool::connect_with(
        options
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .create_if_missing(true),
    )
    .await?;
    if migrate {
        info!("Running database migrations");
        sqlx::migrate!("./migrations").run(&db).await?;
    }
    Ok(db)
}
