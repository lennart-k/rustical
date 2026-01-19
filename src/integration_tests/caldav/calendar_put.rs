use axum::body::Body;
use headers::{Authorization, HeaderMapExt};
use http::{Request, StatusCode};
use rstest::rstest;
use rustical_store::CalendarMetadata;
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
use tower::ServiceExt;

use crate::integration_tests::{
    ResponseExtractString, caldav::calendar::mkcalendar_template, get_app,
};

#[rstest]
#[tokio::test]
async fn test_put_invalid(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let context = context.await;
    let app = get_app(context.clone());

    let calendar_meta = CalendarMetadata {
        displayname: Some("Calendar".to_string()),
        description: Some("Description".to_string()),
        color: Some("#00FF00".to_string()),
        order: 0,
    };
    let (principal, cal_id) = ("user", "calendar");
    let url = format!("/caldav/principal/{principal}/{cal_id}");

    let mut request = Request::builder()
        .method("MKCALENDAR")
        .uri(&url)
        .body(Body::from(mkcalendar_template(&calendar_meta)))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Invalid calendar data
    let ical = r"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Example Corp.//CalDAV Client//EN
BEGIN:VEVENT
UID:20010712T182145Z-123401@example.com
DTSTAMP:20060712T182145Z
DTSTART:20060714T170000Z
RRULE:UNTIL=123
DTEND:20060715T040000Z
SUMMARY:Bastille Day Party
END:VEVENT
END:VCALENDAR";

    let mut request = Request::builder()
        .method("PUT")
        .uri(format!("{url}/qwue23489.ics"))
        .header("If-None-Match", "*")
        .header("Content-Type", "text/calendar")
        .body(Body::from(ical))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
    let body = response.extract_string().await;
    insta::assert_snapshot!(body, @r#"
    <?xml version="1.0" encoding="utf-8"?>
    <error xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CARD="urn:ietf:params:xml:ns:carddav" xmlns:CS="http://calendarserver.org/ns/" xmlns:PUSH="https://bitfire.at/webdav-push">
        <CAL:valid-calendar-data/>
    </error>
    "#);
}
