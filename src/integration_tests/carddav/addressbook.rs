use crate::integration_tests::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::{HeaderValue, StatusCode};
use rstest::rstest;
use rustical_store::{Addressbook, AddressbookStore};
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
use tower::ServiceExt;

fn mkcol_template(displayname: &str, description: &str) -> String {
    format!(
        r#"
<?xml version='1.0' encoding='UTF-8' ?>
<mkcol xmlns="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
    <set>
        <prop>
            <resourcetype>
                <collection />
                <CARD:addressbook />
            </resourcetype>
            <displayname>{displayname}</displayname>
            <CARD:addressbook-description>{description}</CARD:addressbook-description>
        </prop>
    </set>
</mkcol>
    "#,
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
    let addr_store = context.addr_store;

    let (mut displayname, mut description) = (
        Some("Contacts".to_owned()),
        Some("Amazing contacts!".to_owned()),
    );
    let (principal, addr_id) = ("user", "contacts");
    let url = format!("/carddav/principal/{principal}/{addr_id}");

    let request_template = || {
        Request::builder()
            .method("MKCALENDAR")
            .uri(&url)
            .body(Body::from(mkcol_template(
                displayname.as_ref().unwrap(),
                description.as_ref().unwrap(),
            )))
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
            "dav": "1, 3, access-control, addressbook, webdav-push",
            "allow": "PROPFIND, PROPPATCH, COPY, MOVE, DELETE, OPTIONS, REPORT, GET, HEAD, POST, MKCOL, IMPORT",
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
    insta::assert_snapshot!("mkcol_body", body);

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

    let saved_addressbook = addr_store
        .get_addressbook(principal, addr_id, false)
        .await
        .unwrap();
    assert_eq!(
        (saved_addressbook.displayname, saved_addressbook.description),
        (displayname, description)
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
    <propertyupdate xmlns="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
        <set>
            <prop>
                <displayname>New Displayname</displayname>
                <CARD:addressbook-description>Test</CARD:addressbook-description>
            </prop>
        </set>
        <remove>
            <prop>
                <CARD:addressbook-description />
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

    displayname = Some("New Displayname".to_string());
    description = None;
    let saved_addressbook = addr_store
        .get_addressbook(principal, addr_id, false)
        .await
        .unwrap();
    assert_eq!(
        (saved_addressbook.displayname, saved_addressbook.description),
        (displayname, description)
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
        addr_store.get_addressbook(principal, addr_id, false).await,
        Err(rustical_store::Error::NotFound)
    ));
}
