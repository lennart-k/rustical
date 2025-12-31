use crate::{
    SqliteStore, addressbook_store::SqliteAddressbookStore, calendar_store::SqliteCalendarStore,
    principal_store::SqlitePrincipalStore,
};
use rstest::fixture;
use rustical_store::auth::{AuthenticationProvider, Principal, PrincipalType};
use sqlx::SqlitePool;

mod addressbook_store;
mod calendar_store;

async fn get_test_db() -> SqlitePool {
    let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    // Populate with test data
    let principal_store = SqlitePrincipalStore::new(db.clone());
    principal_store
        .insert_principal(
            Principal {
                id: "user".to_owned(),
                displayname: None,
                memberships: vec![],
                password: None,
                principal_type: PrincipalType::Individual,
            },
            false,
        )
        .await
        .unwrap();
    principal_store
        .add_app_token("user", "test".to_string(), "pass".to_string())
        .await
        .unwrap();

    db
}

#[derive(Debug, Clone)]
pub struct TestStoreContext {
    pub db: SqlitePool,
    pub addr_store: SqliteAddressbookStore,
    pub cal_store: SqliteCalendarStore,
    pub principal_store: SqlitePrincipalStore,
    pub sub_store: SqliteStore,
}

#[fixture]
pub async fn test_store_context() -> TestStoreContext {
    let (send_addr, _recv) = tokio::sync::mpsc::channel(1);
    let (send_cal, _recv) = tokio::sync::mpsc::channel(1);
    let db = get_test_db().await;
    TestStoreContext {
        db: db.clone(),
        addr_store: SqliteAddressbookStore::new(db.clone(), send_addr),
        cal_store: SqliteCalendarStore::new(db.clone(), send_cal),
        principal_store: SqlitePrincipalStore::new(db.clone()),
        sub_store: SqliteStore::new(db),
    }
}
