use crate::{app::make_app, config::NextcloudLoginConfig};
use axum::extract::Request;
use axum::{body::Body, response::Response};
use rstest::rstest;
use rustical_caldav::CalDavConfig;
use rustical_frontend::FrontendConfig;
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
use std::sync::Arc;
use tower::ServiceExt;

pub fn get_app(context: TestStoreContext) -> axum::Router {
    let TestStoreContext {
        addr_store,
        cal_store,
        principal_store,
        sub_store,
        ..
    } = context;

    make_app(
        Arc::new(addr_store),
        Arc::new(cal_store),
        Arc::new(sub_store),
        Arc::new(principal_store),
        FrontendConfig {
            enabled: true,
            allow_password_login: true,
        },
        None,
        CalDavConfig::default(),
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
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let app = get_app(context.await);

    let response = app
        .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert!(response.status().is_success());
}

mod caldav;
mod carddav;
