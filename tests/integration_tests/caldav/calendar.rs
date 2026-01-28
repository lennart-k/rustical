use super::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::{HeaderValue, StatusCode};
use rstest::rstest;
use rustical_store::{CalendarMetadata, CalendarStore};
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
use tower::ServiceExt;

pub fn mkcalendar_template(
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
            <CAL:supported-calendar-component-set>
                <CAL:comp name="VEVENT"/>
                <CAL:comp name="VTODO"/>
                <CAL:comp name="VJOURNAL"/>
            </CAL:supported-calendar-component-set>
            <CAL:calendar-timezone><![CDATA[BEGIN:VCALENDAR
PRODID:-//Example Corp.//CalDAV Client//EN
VERSION:2.0
BEGIN:VTIMEZONE
TZID:US/Eastern
LAST-MODIFIED:19870101T000000Z
BEGIN:STANDARD
DTSTART:19671029T020000
RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10
TZOFFSETFROM:-0400
TZOFFSETTO:-0500
TZNAME:Eastern Standard Time (US & Canada)
END:STANDARD
BEGIN:DAYLIGHT
DTSTART:19870405T020000
RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=4
TZOFFSETFROM:-0500
TZOFFSETTO:-0400
TZNAME:Eastern Daylight Time (US & Canada)
END:DAYLIGHT
END:VTIMEZONE
END:VCALENDAR
]]></CAL:calendar-timezone>
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
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let context = context.await;
    let app = get_app(context.clone());
    let cal_store = context.cal_store;

    let mut calendar_meta = CalendarMetadata {
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

    // Try OPTIONS without authentication
    let request = Request::builder()
        .method("OPTIONS")
        .uri(&url)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    insta::assert_debug_snapshot!(response, @r#"
    Response {
        status: 200,
        version: HTTP/1.1,
        headers: {
            "dav": "1, 3, access-control, calendar-access, webdav-push",
            "allow": "PROPFIND, PROPPATCH, COPY, MOVE, DELETE, OPTIONS, REPORT, GET, HEAD, POST, MKCOL, MKCALENDAR, IMPORT",
        },
        body: Body(
            UnsyncBoxBody,
        ),
    }
    "#);

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
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.extract_string().await;
    insta::assert_snapshot!("mkcalendar_body", body);

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
    insta::assert_snapshot!("get_body", body);

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
        insta::assert_snapshot!("propfind_body", body);
    });

    let proppatch_request: &str = r#"
    <propertyupdate xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
        <set>
            <prop>
                <displayname>New Displayname</displayname>
                <CAL:calendar-description>Test</CAL:calendar-description>
            </prop>
        </set>
        <remove>
            <prop>
                <CAL:calendar-description />
            </prop>
        </remove>
    </propertyupdate>
    "#;
    let mut request = Request::builder()
        .method("PROPPATCH")
        .uri(&url)
        .body(Body::from(proppatch_request))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::MULTI_STATUS);
    let body = response.extract_string().await;
    insta::assert_snapshot!("proppatch_body", body);

    calendar_meta.displayname = Some("New Displayname".to_string());
    calendar_meta.description = None;

    assert_eq!(
        cal_store
            .get_calendar(principal, cal_id, false)
            .await
            .unwrap()
            .meta,
        calendar_meta
    );

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
    insta::assert_snapshot!("delete_body", body);

    assert!(matches!(
        cal_store.get_calendar(principal, cal_id, false).await,
        Err(rustical_store::Error::NotFound)
    ));
}

#[rstest]
#[tokio::test]
async fn test_rfc4791_5_3_2(
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

    let request_template = || {
        Request::builder()
            .method("MKCALENDAR")
            .uri(&url)
            .body(Body::from(mkcalendar_template(&calendar_meta)))
            .unwrap()
    };

    // Try with correct credentials
    let mut request = request_template();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let ical = r"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Example Corp.//CalDAV Client//EN
BEGIN:VEVENT
UID:20010712T182145Z-123401@example.com
DTSTAMP:20060712T182145Z
DTSTART:20060714T170000Z
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
    assert_eq!(response.status(), StatusCode::CREATED);

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
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let mut request = Request::builder()
        .method("REPORT")
        .uri(&url)
        .header("Depth", "infinity")
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .body(Body::from(format!(
            r#"
            <?xml version="1.0" encoding="utf-8" ?>
            <C:calendar-multiget xmlns:D="DAV:"
                                    xmlns:C="urn:ietf:params:xml:ns:caldav">
                <D:prop>
                    <D:getetag/>
                </D:prop>
                <D:href>{url}/qwue23489.ics</D:href>
                <D:href>/home/bernard/addressbook/vcf1.vcf</D:href>
            </C:calendar-multiget>
        "#
        )))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::MULTI_STATUS);
    let body = response.extract_string().await;
    insta::assert_snapshot!("multiget_body", body);
}
