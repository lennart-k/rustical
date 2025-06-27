use crate::{
    SqliteStore, addressbook_store::SqliteAddressbookStore, calendar_store::SqliteCalendarStore,
    principal_store::SqlitePrincipalStore,
};
use rustical_store::{
    AddressbookStore, CalendarStore, CollectionOperation, SubscriptionStore,
    auth::AuthenticationProvider,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

pub async fn get_test_stores() -> (
    Arc<impl AddressbookStore>,
    Arc<impl CalendarStore>,
    Arc<impl SubscriptionStore>,
    Arc<impl AuthenticationProvider>,
    Receiver<CollectionOperation>,
) {
    let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    // let db = create_db_pool("sqlite::memory:", true).await.unwrap();
    // Channel to watch for changes (for DAV Push)
    let (send, recv) = tokio::sync::mpsc::channel(1000);

    let addressbook_store = Arc::new(SqliteAddressbookStore::new(db.clone(), send.clone()));
    let cal_store = Arc::new(SqliteCalendarStore::new(db.clone(), send));
    let subscription_store = Arc::new(SqliteStore::new(db.clone()));
    let principal_store = Arc::new(SqlitePrincipalStore::new(db.clone()));
    (
        addressbook_store,
        cal_store,
        subscription_store,
        principal_store,
        recv,
    )
}

#[tokio::test]
async fn test_create_store() {
    get_test_stores().await;
}
