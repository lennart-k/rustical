#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
pub use error::Error;
use serde::Serialize;
use sqlx::{PgPool, Pool, Postgres};
use tracing::info;
pub mod addressbook_store;
pub mod calendar_store;
pub mod error;
pub mod principal_store;
pub mod subscription_store;

#[cfg(any(test, feature = "test"))]
pub mod tests;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
#[repr(i32)]
pub(crate) enum ChangeOperation {
    Add = 0,
    Delete = 1,
}

#[derive(Debug, Clone)]
pub struct PostgresStore {
    db: PgPool,
}

impl PostgresStore {
    #[must_use]
    pub const fn new(db: PgPool) -> Self {
        Self { db }
    }
}

pub async fn create_db_pool(db_url: &str, migrate: bool) -> Result<Pool<Postgres>, sqlx::Error> {
    let db = PgPool::connect(db_url).await?;
    if migrate {
        info!("Running database migrations");
        sqlx::migrate!("./migrations").run(&db).await?;
    }
    Ok(db)
}
