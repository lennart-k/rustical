use crate::integration_tests::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::{HeaderValue, StatusCode};
use rstest::rstest;
use rustical_store::{CalendarMetadata, CalendarStore};
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
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
