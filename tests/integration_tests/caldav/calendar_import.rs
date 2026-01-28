use super::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::StatusCode;
use rstest::rstest;
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
use tower::ServiceExt;

const ICAL: &str = r"
BEGIN:VCALENDAR
PRODID:-//Example Corp.//CalDAV Client//EN
VERSION:2.0
BEGIN:VEVENT
UID:1@example.com
SUMMARY:One-off Meeting
DTSTAMP:20041210T183904Z
DTSTART:20041207T120000Z
DTEND:20041207T130000Z
END:VEVENT
BEGIN:VEVENT
UID:2@example.com
SUMMARY:Weekly Meeting
DTSTAMP:20041210T183838Z
DTSTART:20041206T120000Z
DTEND:20041206T130000Z
RRULE:FREQ=WEEKLY
END:VEVENT
BEGIN:VEVENT
UID:2@example.com
SUMMARY:Weekly Meeting
RECURRENCE-ID:20041213T120000Z
DTSTAMP:20041210T183838Z
DTSTART:20041213T130000Z
DTEND:20041213T140000Z
END:VEVENT
END:VCALENDAR
";

#[rstest]
#[case(0, ICAL)]
#[case(1, include_str!("resources/rfc4791_appb.ics"))]
#[tokio::test]
async fn test_import(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
    #[case] case: usize,
    #[case] ical: &'static str,
) {
    let context = context.await;
    let app = get_app(context.clone());

    let (principal, addr_id) = ("user", "calendar");
    let url = format!("/caldav/principal/{principal}/{addr_id}");

    let request_template = || {
        Request::builder()
            .method("IMPORT")
            .uri(&url)
            .body(Body::from(ical))
            .unwrap()
    };

    // Try without authentication
    let request = request_template();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Try with correct credentials
    let mut request = request_template();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.extract_string().await;
    insta::assert_snapshot!(format!("{case}_import_body"), body);

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
    insta::with_settings!({
        filters => vec![
            (r"UID:.+", "UID:[UID]")
        ]
    }, {
        insta::assert_snapshot!(format!("{case}_get_body"), body);
    });
}
