use crate::integration_tests::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::{HeaderValue, StatusCode};
use rstest::rstest;
use rustical_store::{CalendarMetadata, CalendarStore};
use rustical_store_sqlite::{calendar_store::SqliteCalendarStore, tests::get_test_calendar_store};
use tower::ServiceExt;

fn mkcalendar_template(
    CalendarMetadata {
        displayname,
        order: _order,
        description,
        color,
    }: &CalendarMetadata,
) -> String {
    format!(
        r#"
<?xml version='1.0' encoding='UTF-8' ?>
<CAL:mkcalendar xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
    <set>
        <prop>
            <resourcetype>
                <collection />
                <CAL:calendar />
            </resourcetype>
            <displayname>{displayname}</displayname>
            <CAL:calendar-description>{description}</CAL:calendar-description>
            <n0:calendar-color xmlns:n0="http://apple.com/ns/ical/">{color}</n0:calendar-color>
            <CAL:calendar-timezone-id>Europe/Berlin</CAL:calendar-timezone-id>
            <CAL:supported-calendar-component-set>
                <CAL:comp name="VEVENT"/>
                <CAL:comp name="VTODO"/>
                <CAL:comp name="VJOURNAL"/>
            </CAL:supported-calendar-component-set>
        </prop>
    </set>
</CAL:mkcalendar>
    "#,
        displayname = displayname.as_deref().unwrap_or_default(),
        description = description.as_deref().unwrap_or_default(),
        color = color.as_deref().unwrap_or_default(),
    )
}

#[rstest]
#[tokio::test]
async fn test_caldav_calendar(
    #[from(get_app)]
    #[future]
    app: axum::Router,
    #[from(get_test_calendar_store)]
    #[future]
    cal_store: SqliteCalendarStore,
) {
    let app = app.await;
    let cal_store = cal_store.await;

    let calendar_meta = CalendarMetadata {
        displayname: Some("Calendar".to_string()),
        description: Some("Description".to_string()),
        color: Some("#00FF00".to_string()),
        order: 0,
    };
    let (principal, cal_id) = ("user", "calendar");
    let url = format!("/caldav/principal/{principal}/{cal_id}");

    let request_template = || {
        Request::builder()
            .method("MKCALENDAR")
            .uri(&url)
            .body(Body::from(mkcalendar_template(&calendar_meta)))
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
        .uri(&url)
        .body(Body::empty())
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.extract_string().await;
    insta::assert_snapshot!(body);

    assert_eq!(
        cal_store
            .get_calendar(principal, cal_id, false)
            .await
            .unwrap()
            .meta,
        calendar_meta
    );

    let mut request = Request::builder()
        .method("PROPFIND")
        .uri(&url)
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
            (r"<PUSH:topic>[0-9a-f-]+</PUSH:topic>", "<PUSH:topic>[PUSH_TOPIC]</PUSH:topic>")
        ]
    }, {
        insta::assert_snapshot!(body);
    });

    let mut request = Request::builder()
        .method("DELETE")
        .uri(&url)
        .header("X-No-Trashbin", HeaderValue::from_static("1"))
        .body(Body::empty())
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.extract_string().await;
    insta::assert_snapshot!(body);

    assert!(matches!(
        cal_store.get_calendar(principal, cal_id, false).await,
        Err(rustical_store::Error::NotFound)
    ));
}

