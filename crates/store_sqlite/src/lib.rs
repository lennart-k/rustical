pub use error::Error;
use serde::Serialize;
use sqlx::{Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};
use tracing::info;
pub mod addressbook_store;
pub mod calendar_store;
pub mod error;
pub mod principal_store;
pub mod subscription_store;

#[derive(Debug, Clone, Serialize, sqlx::Type)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ChangeOperation {
    // There's no distinction between Add and Modify
    Add,
    Delete,
}

#[derive(Debug)]
pub struct SqliteStore {
    db: SqlitePool,
}

impl SqliteStore {
    pub fn new(db: SqlitePool) -> Self {
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

pub async fn create_test_db() -> Result<SqlitePool, sqlx::Error> {
    let db = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    Ok(db)
}

pub async fn create_test_store() -> Result<SqliteStore, sqlx::Error> {
    Ok(SqliteStore::new(create_test_db().await?))
}
