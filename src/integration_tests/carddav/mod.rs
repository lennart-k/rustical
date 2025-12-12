use crate::integration_tests::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::StatusCode;
use rstest::rstest;
use tower::ServiceExt;

#[rstest]
#[tokio::test]
async fn test_carddav_root(
    #[from(get_app)]
    #[future]
    app: axum::Router,
) {
    let app = app.await;

    let request_template = || {
        Request::builder()
            .method("PROPFIND")
            .uri("/carddav")
            .body(Body::empty())
            .unwrap()
    };

    // Try without authentication
    let request = request_template();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response.extract_string().await;
    insta::assert_snapshot!(body);

    // Try with wrong password
    let mut request = request_template();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "wrongpass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response.extract_string().await;
    insta::assert_snapshot!(body);

    // Try with correct credentials
    let mut request = request_template();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::MULTI_STATUS);
    let body = response.extract_string().await;
    insta::assert_snapshot!(body);
}
