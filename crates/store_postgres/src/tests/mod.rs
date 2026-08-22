use crate::{
    PostgresStore, addressbook_store::PostgresAddressbookStore,
    calendar_store::PostgresCalendarStore, principal_store::PostgresPrincipalStore,
};
use rstest::{fixture, rstest};
use rustical_store::auth::{AuthenticationProvider, Principal, PrincipalType};
use sqlx::PgPool;
use sqlx::postgres::PgConnectOptions;
use std::str::FromStr;

mod addressbook_store;
mod calendar_store;

#[derive(Debug, Clone)]
pub struct TestStoreContext {
    pub db: PgPool,
    pub addr_store: PostgresAddressbookStore,
    pub cal_store: PostgresCalendarStore,
    pub principal_store: PostgresPrincipalStore,
    pub sub_store: PostgresStore,
}

async fn test_pool() -> PgPool {
    let url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL required for postgres store tests");
    let admin = PgPool::connect(&url).await.expect("connect postgres");
    let schema = format!("t_{}", uuid::Uuid::new_v4().simple());
    sqlx::query(sqlx::AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
        .execute(&admin)
        .await
        .expect("create test schema");
    drop(admin);

    let opts = PgConnectOptions::from_str(&url)
        .expect("parse DATABASE_URL")
        .options([("search_path", schema.as_str())]);
    let db = PgPool::connect_with(opts).await.expect("connect schema");
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("migrate");
    db
}

#[fixture]
pub async fn test_store_context() -> TestStoreContext {
    let (send_addr, _recv) = tokio::sync::mpsc::channel(1);
    let (send_cal, _recv) = tokio::sync::mpsc::channel(1);
    let db = test_pool().await;

    let principal_store = PostgresPrincipalStore::new(db.clone());
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
        addr_store: PostgresAddressbookStore::new(db.clone(), send_addr, false),
        cal_store: PostgresCalendarStore::new(db.clone(), send_cal, false),
        principal_store,
        sub_store: PostgresStore::new(db),
    }
}

#[rstest]
#[tokio::test]
async fn test_invalid_principal_id(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let principal_store = context.await.principal_store;

    assert!(
        matches!(
            principal_store
                .insert_principal(
                    Principal {
                        id: "group:nicegroup".to_owned(),
                        displayname: None,
                        principal_type: PrincipalType::Individual,
                        password: None,
                        memberships: vec![],
                    },
                    false,
                )
                .await,
            Err(rustical_store::Error::InvalidPrincipalId)
        ),
        ": not allowed since it breaks basic auth"
    );

    assert!(
        matches!(
            principal_store
                .insert_principal(
                    Principal {
                        id: "nice$user".to_owned(),
                        displayname: None,
                        principal_type: PrincipalType::Individual,
                        password: None,
                        memberships: vec![],
                    },
                    false,
                )
                .await,
            Err(rustical_store::Error::InvalidPrincipalId)
        ),
        ": not allowed since '$' symbol is reserved for principal impersonation"
    );
}
