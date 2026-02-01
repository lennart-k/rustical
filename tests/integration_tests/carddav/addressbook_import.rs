use super::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::{HeaderValue, StatusCode};
use rstest::rstest;
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
use tower::ServiceExt;

#[rstest]
#[tokio::test]
async fn test_import(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let context = context.await;
    let app = get_app(context.clone());

    let (principal, addr_id) = ("user", "contacts");
    let url = format!("/carddav/principal/{principal}/{addr_id}");
    let bday_url = format!("/caldav/principal/{principal}/_birthdays_{addr_id}");

    let request_template = || {
        Request::builder()
            .method("IMPORT")
            .uri(&url)
            .body(Body::from(
                r"BEGIN:VCARD
VERSION:4.0
FN:John Doe
N:Doe;John;;;,
BDAY:--0203
ANNIVERSARY:--0303
GENDER:M
UID:amazing-uid
END:VCARD",
            ))
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
    insta::assert_snapshot!("import_body", body);

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

    // Create birthday calendar
    let mut request = Request::builder().method("MKCOL").uri(&bday_url).body(
        Body::from(r#"
      <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>Test Birthdays</displayname>
            <CAL:calendar-description>and anniversaries</CAL:calendar-description>
            <ICAL:calendar-color>#FFFF00</ICAL:calendar-color>
            <CAL:supported-calendar-component-set>
              <CAL:comp name="VEVENT" />
            </CAL:supported-calendar-component-set>
          </prop>
        </set>
      </mkcol>
        "#)
    ).unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Get birthday objects
    let mut request = Request::builder()
        .method("GET")
        .uri(&bday_url)
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
            (r"DTSTAMP:[0-9\-:TZ=;]+", "DTSTAMP:[DTSTAMP]")
        ]
    }, {
        insta::assert_snapshot!("birthdays_body", body);
    });

    let mut request = Request::builder()
        .method("PROPFIND")
        .uri(&bday_url)
        .body(Body::empty())
        .unwrap();
    request
        .headers_mut()
        .insert("Depth", HeaderValue::from_static("1"));
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::MULTI_STATUS);
    let body = response.extract_string().await;
    insta::with_settings!({
        filters => vec![
            (r"DTSTAMP:[0-9\-:TZ=;]+", "DTSTAMP:[DTSTAMP]"),
            (r"<PUSH:topic>[0-9a-f-]+</PUSH:topic>", "<PUSH:topic>[PUSH_TOPIC]</PUSH:topic>"),
            (r#"<getetag>&quot;[0-9a-f-]+&quot;</getetag>"#, r#"<getetag>&quot;[GETETAG]&quot;</getetag>"#)
        ]
    }, {
        insta::assert_snapshot!("birthdays_propfind", body);
    });
}
