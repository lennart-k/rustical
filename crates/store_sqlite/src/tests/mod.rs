use crate::{
    SqliteStore, addressbook_store::SqliteAddressbookStore, calendar_store::SqliteCalendarStore,
    create_db_pool, principal_store::SqlitePrincipalStore,
};
use rstest::fixture;
use rustical_store::auth::{AuthenticationProvider, Principal, PrincipalType};
use sqlx::SqlitePool;

mod addressbook_store;
mod calendar_store;

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
    let db = create_db_pool(":memory:", true).await.unwrap();

    let principal_store = SqlitePrincipalStore::new(db.clone());
    // Populate with test data
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

    TestStoreContext {
        db: db.clone(),
        addr_store: SqliteAddressbookStore::new(db.clone(), send_addr, false),
        cal_store: SqliteCalendarStore::new(db.clone(), send_cal, false),
        principal_store,
        sub_store: SqliteStore::new(db),
    }
}
