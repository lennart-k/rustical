use crate::{
    SqliteStore, addressbook_store::SqliteAddressbookStore, calendar_store::SqliteCalendarStore,
    principal_store::SqlitePrincipalStore,
};
use rustical_store::{
    Secret,
    auth::{AuthenticationProvider, Principal, PrincipalType},
};
use sqlx::SqlitePool;
use tokio::sync::OnceCell;

static DB: OnceCell<SqlitePool> = OnceCell::const_new();

mod addressbook_store;
mod calendar_store;

async fn get_test_db() -> SqlitePool {
    DB.get_or_init(async || {
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
    })
    .await
    .clone()
}

#[rstest::fixture]
pub async fn get_test_addressbook_store() -> SqliteAddressbookStore {
    let (send, _recv) = tokio::sync::mpsc::channel(1000);
    SqliteAddressbookStore::new(get_test_db().await, send)
}
#[rstest::fixture]
pub async fn get_test_calendar_store() -> SqliteCalendarStore {
    let (send, _recv) = tokio::sync::mpsc::channel(1000);
    SqliteCalendarStore::new(get_test_db().await, send)
}
#[rstest::fixture]
pub async fn get_test_subscription_store() -> SqliteStore {
    SqliteStore::new(get_test_db().await)
}
#[rstest::fixture]
pub async fn get_test_principal_store() -> SqlitePrincipalStore {
    SqlitePrincipalStore::new(get_test_db().await)
}
