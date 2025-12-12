use crate::integration_tests::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::{HeaderValue, StatusCode};
use rstest::rstest;
use tower::ServiceExt;

mod calendar;

#[rstest]
#[tokio::test]
async fn test_caldav_root(
    #[from(get_app)]
    #[future]
    app: axum::Router,
) {
    let app = app.await;

    let request_template = || {
        Request::builder()
            .method("PROPFIND")
            .uri("/caldav")
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

#[rstest]
#[tokio::test]
async fn test_caldav_principal(
    #[from(get_app)]
    #[future]
    app: axum::Router,
) {
    let app = app.await;

    let request_template = || {
        Request::builder()
            .method("PROPFIND")
            .uri("/caldav/principal/user")
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
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::MULTI_STATUS);
    let body = response.extract_string().await;
    insta::assert_snapshot!(body);

    // Try with Depth: 1
    let mut request = request_template();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    request
        .headers_mut()
        .insert("Depth", HeaderValue::from_static("1"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::MULTI_STATUS);
    let body = response.extract_string().await;
    insta::with_settings!({
        filters => vec![
            (r"<PUSH:topic>[0-9a-f-]+</PUSH:topic>", "[PUSH_TOPIC]")
        ]
    }, {
        insta::assert_snapshot!(body);
    });
}
