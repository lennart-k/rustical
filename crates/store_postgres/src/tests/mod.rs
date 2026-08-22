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
        .options([("search_path", schema.as_str()), ("TimeZone", "UTC")]);
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

#[rstest]
#[tokio::test]
async fn test_app_tokens_and_no_overwrite(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let principal_store = context.await.principal_store;
    let tokens = principal_store.get_app_tokens("user").await.unwrap();
    assert_eq!(tokens.len(), 1);
    assert!(tokens[0].created_at.is_some());

    principal_store
        .insert_principal(
            Principal {
                id: "user".to_owned(),
                displayname: None,
                memberships: vec![],
                password: Some("keep".to_owned().into()),
                principal_type: PrincipalType::Individual,
            },
            true,
        )
        .await
        .unwrap();
    assert!(matches!(
        principal_store
            .insert_principal(
                Principal {
                    id: "user".to_owned(),
                    displayname: None,
                    memberships: vec![],
                    password: Some("clobber".to_owned().into()),
                    principal_type: PrincipalType::Individual,
                },
                false,
            )
            .await,
        Err(rustical_store::Error::AlreadyExists)
    ));
    assert_eq!(
        principal_store
            .get_principal("user")
            .await
            .unwrap()
            .unwrap()
            .password
            .unwrap()
            .into_inner(),
        "keep"
    );
}

#[rstest]
#[tokio::test]
async fn test_subscription_store(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    use rustical_dav_push::{Subscription, SubscriptionStore};

    let sub_store = context.await.sub_store;
    let sub = Subscription {
        id: "s1".into(),
        topic: "t".into(),
        expiration: chrono::Utc::now().naive_utc(),
        push_resource: "https://example".into(),
        public_key: "k".into(),
        public_key_type: "p256dh".into(),
        auth_secret: "a".into(),
    };
    assert!(!sub_store.upsert_subscription(sub).await.unwrap());
    assert_eq!(sub_store.get_subscription("s1").await.unwrap().topic, "t");
}
