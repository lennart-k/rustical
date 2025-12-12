use crate::{app::make_app, config::NextcloudLoginConfig};
use axum::extract::Request;
use axum::{body::Body, response::Response};
use rstest::rstest;
use rustical_frontend::FrontendConfig;
use rustical_store_sqlite::{
    SqliteStore,
    addressbook_store::SqliteAddressbookStore,
    calendar_store::SqliteCalendarStore,
    principal_store::SqlitePrincipalStore,
    tests::{
        get_test_addressbook_store, get_test_calendar_store, get_test_principal_store,
        get_test_subscription_store,
    },
};
use std::sync::Arc;
use tower::ServiceExt;

#[rstest::fixture]
pub async fn get_app(
    #[from(get_test_calendar_store)]
    #[future]
    cal_store: SqliteCalendarStore,
    #[from(get_test_addressbook_store)]
    #[future]
    addr_store: SqliteAddressbookStore,
    #[from(get_test_principal_store)]
    #[future]
    principal_store: SqlitePrincipalStore,
    #[from(get_test_subscription_store)]
    #[future]
    sub_store: SqliteStore,
) -> axum::Router {
    let addr_store = Arc::new(addr_store.await);
    let cal_store = Arc::new(cal_store.await);
    let sub_store = Arc::new(sub_store.await);
    let principal_store = Arc::new(principal_store.await);

    make_app(
        addr_store,
        cal_store,
        sub_store,
        principal_store,
        FrontendConfig {
            enabled: true,
            allow_password_login: true,
        },
        None,
        &NextcloudLoginConfig { enabled: false },
        false,
        true,
        20,
    )
}

pub trait ResponseExtractString {
    #[allow(async_fn_in_trait)]
    async fn extract_string(self) -> String;
}

impl ResponseExtractString for Response {
    async fn extract_string(self) -> String {
        let bytes = axum::body::to_bytes(self.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}

#[rstest]
#[tokio::test]
async fn test_ping(
    #[from(get_app)]
    #[future]
    app: axum::Router,
) {
    let app = app.await;

    let response = app
        .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert!(response.status().is_success());
}

mod caldav;
mod carddav;
