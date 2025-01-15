use serde::Serialize;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};

pub mod addressbook_store;
pub mod calendar_store;
pub mod error;
pub mod subscription_store;

pub use error::Error;

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
        println!("Running database migrations");
        sqlx::migrate!("./migrations").run(&db).await?;
    }
    Ok(db)
}

pub async fn create_test_store() -> Result<SqliteStore, sqlx::Error> {
    let db = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    Ok(SqliteStore::new(db))
}
