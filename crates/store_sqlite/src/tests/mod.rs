use crate::SqliteStore;
use sqlx::SqlitePool;

pub async fn create_test_db() -> Result<SqlitePool, sqlx::Error> {
    let db = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    Ok(db)
}

#[tokio::test]
async fn test_create_store() {
    SqliteStore::new(create_test_db().await.unwrap());
}
