use super::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::{HeaderValue, StatusCode};
use rstest::rstest;
use rustical_store::AddressbookReadStore;
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
#[case("plain.vcf")]
#[case("space%20name.vcf")]
#[case("question%3Fmark.vcf")]
#[case("hash%23name.vcf")]
#[case("slash%2Fname.vcf")]
#[case("Ren%C3%A9.vcf")]
#[tokio::test]
async fn test_carddav_multiget_encoded_hrefs(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
    #[case] filename: &str,
) {
    let app = get_app(context.await);
    let collection = "/carddav/principal/user/contacts";
    let href = format!("{collection}/{filename}");
    let missing_href = format!("{collection}/missing-{filename}");
    let vcard =
        "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:multiget-test\r\nFN:Test Contact\r\nEND:VCARD\r\n";

    for (method, uri, body, expected_status) in [
        (
            "MKCOL",
            collection,
            mkcol_template("Contacts", "Test contacts"),
            StatusCode::CREATED,
        ),
        ("PUT", href.as_str(), vcard.to_owned(), StatusCode::CREATED),
    ] {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::from(body))
            .unwrap();
        request
            .headers_mut()
            .typed_insert(Authorization::basic("user", "pass"));
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), expected_status);
    }

    let mut request = Request::builder()
        .method("REPORT")
        .uri(collection)
        .body(Body::from(format!(
            r#"<CARD:addressbook-multiget xmlns:D="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
                <D:prop><D:getetag /></D:prop>
                <D:href>{href}</D:href>
                <D:href>{missing_href}</D:href>
            </CARD:addressbook-multiget>"#
        )))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::MULTI_STATUS);
    let body = response.extract_string().await;
    assert!(body.contains(&format!("<href>{href}</href>")), "{body}");
    assert!(
        body.contains(&format!("<href>{missing_href}</href>")),
        "{body}"
    );
    assert!(body.contains("HTTP/1.1 404 Not Found"), "{body}");

    let returned_href = body
        .split_once("<href>")
        .unwrap()
        .1
        .split_once("</href>")
        .unwrap()
        .0;
    let mut request = Request::builder()
        .uri(returned_href)
        .body(Body::empty())
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.extract_string().await.contains("FN:Test Contact"));
}

#[rstest]
#[case("/carddav/principal/user/contacts/bad name.vcf")]
#[case("http://[invalid")]
#[tokio::test]
async fn test_carddav_multiget_invalid_hrefs(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
    #[case] href: &str,
) {
    let app = get_app(context.await);
    let mut request = Request::builder()
        .method("REPORT")
        .uri("/carddav/principal/user/contacts")
        .body(Body::from(format!(
            r#"<CARD:addressbook-multiget xmlns:D="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
                <D:prop><D:getetag /></D:prop>
                <D:href>{href}</D:href>
            </CARD:addressbook-multiget>"#
        )))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[rstest]
#[tokio::test]
async fn test_carddav_addressbook(
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
            .method("MKCOL")
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
            (r"<PUSH:topic>[0-9a-f-]+</PUSH:topic>", "<PUSH:topic>[PUSH_TOPIC]</PUSH:topic>"),
            (
                r#"<PUSH:vapid-public-key type=\"p256ecdsa\">.*</PUSH:vapid-public-key>"#,
                r#"<PUSH:vapid-public-key type="p256ecdsa">[pubkey]</PUSH:vapid-public-key>"#,
            ),
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

#[rstest]
#[tokio::test]
async fn test_mkcol_rfc6352_6_3_1_1(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let context = context.await;
    let app = get_app(context.clone());
    let addr_store = context.addr_store;

    let (displayname, description) = (
        "Lisa's Contacts".to_owned(),
        "My primary address book.".to_owned(),
    );
    let (principal, addr_id) = ("user", "contacts");
    let url = format!("/carddav/principal/{principal}/{addr_id}");

    let mut request = Request::builder()
        .method("MKCOL")
        .uri(&url)
        .body(Body::from(format!(
            r#"<?xml version="1.0" encoding="utf-8" ?>
   <D:mkcol xmlns:D="DAV:"
                 xmlns:C="urn:ietf:params:xml:ns:carddav">
     <D:set>
       <D:prop>
         <D:resourcetype>
           <D:collection/>
           <C:addressbook/>
         </D:resourcetype>
         <D:displayname>{displayname}</D:displayname>
         <C:addressbook-description xml:lang="en"
   >{description}</C:addressbook-description>
       </D:prop>
     </D:set>
   </D:mkcol>"#
        )))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.extract_string().await;
    insta::assert_snapshot!("mkcol_body", body);
    let saved_addressbook = addr_store
        .get_addressbook(principal, addr_id, false)
        .await
        .unwrap();
    assert_eq!(
        (
            saved_addressbook.displayname.unwrap(),
            saved_addressbook.description.unwrap()
        ),
        (displayname, description)
    );

    let vcard = r"BEGIN:VCARD
VERSION:3.0
FN:Cyrus Daboo
N:Daboo;Cyrus
ADR;TYPE=POSTAL:;2822 Email HQ;Suite 2821;RFCVille;PA;15213;USA
EMAIL;TYPE=INTERNET,PREF:cyrus@example.com
NICKNAME:me
NOTE:Example VCard.
ORG:Self Employed
TEL;TYPE=WORK,VOICE:412 605 0499
TEL;TYPE=FAX:412 605 0705
URL:http://www.example.com
UID:1234-5678-9000-1
END:VCARD
        ";

    let mut request = Request::builder()
        .method("PUT")
        .uri(format!("{url}/newcard.vcf"))
        .header("If-None-Match", "*")
        .header("Content-Type", "text/vcard")
        .body(Body::from(vcard))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let etag = response.headers().get("ETag").unwrap();

    // This should overwrite
    let mut request = Request::builder()
        .method("PUT")
        .uri(format!("{url}/newcard.vcf"))
        .header("If-None-Match", "\"somearbitraryetag\"")
        .header("Content-Type", "text/vcard")
        .body(Body::from(vcard))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let mut request = Request::builder()
        .method("PUT")
        .uri(format!("{url}/newcard.vcf"))
        .header("If-None-Match", etag)
        .header("Content-Type", "text/vcard")
        .body(Body::from(vcard))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);

    let mut request = Request::builder()
        .method("PUT")
        .uri(format!("{url}/newcard.vcf"))
        .header("If-None-Match", "*")
        .header("Content-Type", "text/vcard")
        .body(Body::from(vcard))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
}

#[rstest]
#[tokio::test]
async fn test_rfc6352_8_7_1(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let context = context.await;
    let app = get_app(context.clone());
    let addr_store = context.addr_store;

    let (displayname, description) = (
        "Lisa's Contacts".to_owned(),
        "My primary address book.".to_owned(),
    );
    let (principal, addr_id) = ("user", "contacts");
    let url = format!("/carddav/principal/{principal}/{addr_id}");

    let mut request = Request::builder()
        .method("MKCOL")
        .uri(&url)
        .body(Body::from(format!(
            r#"<?xml version="1.0" encoding="utf-8" ?>
   <D:mkcol xmlns:D="DAV:"
                 xmlns:C="urn:ietf:params:xml:ns:carddav">
     <D:set>
       <D:prop>
         <D:resourcetype>
           <D:collection/>
           <C:addressbook/>
         </D:resourcetype>
         <D:displayname>{displayname}</D:displayname>
         <C:addressbook-description xml:lang="en"
   >{description}</C:addressbook-description>
       </D:prop>
     </D:set>
   </D:mkcol>"#
        )))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.extract_string().await;
    insta::assert_snapshot!("mkcol_body", body);
    let saved_addressbook = addr_store
        .get_addressbook(principal, addr_id, false)
        .await
        .unwrap();
    assert_eq!(
        (
            saved_addressbook.displayname.unwrap(),
            saved_addressbook.description.unwrap()
        ),
        (displayname, description)
    );

    let vcard = r"BEGIN:VCARD
VERSION:3.0
FN:Cyrus Daboo
N:Daboo;Cyrus
ADR;TYPE=POSTAL:;2822 Email HQ;Suite 2821;RFCVille;PA;15213;USA
EMAIL;TYPE=INTERNET,PREF:cyrus@example.com
NICKNAME:me
NOTE:Example VCard.
ORG:Self Employed
TEL;TYPE=WORK,VOICE:412 605 0499
TEL;TYPE=FAX:412 605 0705
URL:http://www.example.com
UID:1234-5678-9000-1
END:VCARD
        ";

    let mut request = Request::builder()
        .method("PUT")
        .uri(format!("{url}/newcard.vcf"))
        .header("If-None-Match", "*")
        .header("Content-Type", "text/vcard")
        .body(Body::from(vcard))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let mut request = Request::builder()
        .method("REPORT")
        .uri(&url)
        .header("Depth", "infinity")
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .body(Body::from(format!(
            r#"
            <?xml version="1.0" encoding="utf-8" ?>
            <C:addressbook-multiget xmlns:D="DAV:"
                                    xmlns:C="urn:ietf:params:xml:ns:carddav">
                <D:prop>
                <D:getetag/>
                <C:address-data>
                    <C:prop name="VERSION"/>
                    <C:prop name="UID"/>
                    <C:prop name="NICKNAME"/>
                    <C:prop name="EMAIL"/>
                    <C:prop name="FN"/>
                </C:address-data>
                </D:prop>
                <D:href>{url}/newcard.vcf</D:href>
                <D:href>/home/bernard/addressbook/vcf1.vcf</D:href>
            </C:addressbook-multiget>
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
