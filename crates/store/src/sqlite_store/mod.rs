use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};

pub mod addressbook_store;
pub mod calendar_store;

#[derive(Debug)]
pub struct SqliteStore {
    db: SqlitePool,
}

impl SqliteStore {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

pub async fn create_db_pool(db_url: &str, migrate: bool) -> anyhow::Result<Pool<Sqlite>> {
    let db = SqlitePool::connect_with(
        SqliteConnectOptions::new()
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

pub async fn create_test_store() -> anyhow::Result<SqliteStore> {
    let db = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    Ok(SqliteStore::new(db))
}
