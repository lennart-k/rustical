use crate::integration_tests::{ResponseExtractString, get_app};
use axum::body::{Body, Bytes};
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::{HeaderValue, StatusCode};
use rstest::rstest;
use rustical_store::{Calendar, CalendarMetadata};
use tower::ServiceExt;

const MKCOL_REQUEST: &str = r#"
<?xml version='1.0' encoding='UTF-8' ?>
<CAL:mkcalendar xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
    <set>
        <prop>
            <resourcetype>
                <collection />
                <CAL:calendar />
            </resourcetype>
            <displayname>Amazing Calendar</displayname>
            <CAL:calendar-description>Description</CAL:calendar-description>
            <n0:calendar-color xmlns:n0="http://apple.com/ns/ical/">#FFF8DCFF</n0:calendar-color>
            <CAL:calendar-timezone-id>Europe/Berlin</CAL:calendar-timezone-id>
            <CAL:supported-calendar-component-set>
                <CAL:comp name="VEVENT"/>
                <CAL:comp name="VTODO"/>
                <CAL:comp name="VJOURNAL"/>
            </CAL:supported-calendar-component-set>
        </prop>
    </set>
</CAL:mkcalendar>
    "#;

#[rstest]
#[tokio::test]
async fn test_caldav_calendar(
    #[from(get_app)]
    #[future]
    app: axum::Router,
) {
    let app = app.await;

    let request_template = || {
        Request::builder()
            .method("MKCALENDAR")
            .uri("/caldav/principal/user/calendar")
            .body(Body::from(MKCOL_REQUEST))
            .unwrap()
    };

    // Try without authentication
    let request = request_template();
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
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.extract_string().await;
    insta::assert_snapshot!(body);

    let mut request = Request::builder()
        .method("GET")
        .uri("/caldav/principal/user/calendar")
        .body(Body::empty())
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.extract_string().await;
    insta::assert_snapshot!(body);

    let mut request = Request::builder()
        .method("PROPFIND")
        .uri("/caldav/principal/user/calendar")
        .body(Body::empty())
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
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

    let mut request = Request::builder()
        .method("DELETE")
        .uri("/caldav/principal/user/calendar")
        .body(Body::empty())
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.extract_string().await;
    insta::assert_snapshot!(body);
}
