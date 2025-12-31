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
}

pub async fn create_db_pool(db_url: &str, migrate: bool) -> Result<Pool<Sqlite>, sqlx::Error> {
    let db = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .filename(db_url)
            .create_if_missing(true),
    )
    .await?;
    if migrate {
        info!("Running database migrations");
        sqlx::migrate!("./migrations").run(&db).await?;
    }
    Ok(db)
}
