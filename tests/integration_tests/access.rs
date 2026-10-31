//! Who may use which route: the owner, another authenticated principal, nobody.
use super::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::StatusCode;
use rstest::rstest;
use rustical_store::auth::{AuthenticationProvider, Principal, PrincipalType};
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
use tower::ServiceExt;

pub const USER: (&str, &str) = ("user", "pass");
pub const OTHER: (&str, &str) = ("other", "otherpass");

pub const MKCALENDAR: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<C:mkcalendar xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:set><D:prop><D:displayname>Calendar</D:displayname></D:prop></D:set>
</C:mkcalendar>"#;

pub const MKCOL_ADDRESSBOOK: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<mkcol xmlns="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
  <set><prop>
    <resourcetype><collection /><CARD:addressbook /></resourcetype>
    <displayname>Book</displayname>
  </prop></set>
</mkcol>"#;

pub fn event(uid: &str, summary: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\nUID:{uid}\r\n\
         DTSTAMP:20260101T000000Z\r\nDTSTART:20260105T100000Z\r\nDTEND:20260105T110000Z\r\n\
         SUMMARY:{summary}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

pub fn vcard(uid: &str, name: &str) -> String {
    format!("BEGIN:VCARD\r\nVERSION:3.0\r\nUID:{uid}\r\nFN:{name}\r\nEND:VCARD\r\n")
}

pub const PROPPATCH_DISPLAYNAME: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propertyupdate xmlns:D="DAV:">
  <D:set><D:prop><D:displayname>Renamed</D:displayname></D:prop></D:set>
</D:propertyupdate>"#;

pub const PUSH_REGISTER: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<push-register xmlns="https://bitfire.at/webdav-push" xmlns:D="DAV:">
  <subscription>
    <web-push-subscription>
      <push-resource>https://up.example.net/yohd4yai5Phiz1wi</push-resource>
      <subscription-public-key type="p256dh">BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4</subscription-public-key>
      <auth-secret>BTBZMqHH6r4Tts7J_aSIgg</auth-secret>
    </web-push-subscription>
  </subscription>
</push-register>"#;

pub fn sync_collection(token: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8" ?>
<D:sync-collection xmlns:D="DAV:">
  <D:sync-token>{token}</D:sync-token>
  <D:sync-level>1</D:sync-level>
  <D:prop><D:getetag/></D:prop>
</D:sync-collection>"#
    )
}

pub const CALENDAR_QUERY: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<C:calendar-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:prop><D:getetag/><C:calendar-data/></D:prop>
  <C:filter><C:comp-filter name="VCALENDAR"/></C:filter>
</C:calendar-query>"#;

/// Sends one request and returns status and body.
pub async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    auth: Option<(&str, &str)>,
    headers: &[(&str, &str)],
    body: &str,
) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let mut request = builder.body(Body::from(body.to_owned())).unwrap();
    if let Some((user, pass)) = auth {
        request
            .headers_mut()
            .typed_insert(Authorization::basic(user, pass));
    }
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    (status, response.extract_string().await)
}

/// Adds a principal with an app token (the password `pass` of the test context's `user`).
pub async fn add_principal(
    context: &TestStoreContext,
    id: &str,
    principal_type: PrincipalType,
    token: Option<&str>,
) {
    context
        .principal_store
        .insert_principal(
            Principal {
                id: id.to_owned(),
                displayname: Some(format!("{id} display")),
                principal_type,
                password: None,
                memberships: vec![],
            },
            false,
        )
        .await
        .unwrap();
    if let Some(token) = token {
        context
            .principal_store
            .add_app_token(id, "test".to_owned(), token.to_owned())
            .await
            .unwrap();
    }
}

/// `user` gets the calendar `cal` with `obj.ics` and the addressbook `book` with `card.vcf`.
pub async fn seed_user(app: &axum::Router) {
    for (method, uri, body) in [
        (
            "MKCALENDAR",
            "/caldav/principal/user/cal",
            MKCALENDAR.to_owned(),
        ),
        (
            "PUT",
            "/caldav/principal/user/cal/obj.ics",
            event("obj-uid", "Secret"),
        ),
        (
            "MKCOL",
            "/carddav/principal/user/book",
            MKCOL_ADDRESSBOOK.to_owned(),
        ),
        (
            "PUT",
            "/carddav/principal/user/book/card.vcf",
            vcard("card-uid", "Card"),
        ),
    ] {
        let (status, body) = send(app, method, uri, Some(USER), &[], &body).await;
        assert!(status.is_success(), "{method} {uri}: {status} {body}");
    }
}

/// One route of the matrix: method, path, body, extra headers, status for the owner and for
/// another principal.
struct Route {
    method: &'static str,
    uri: &'static str,
    body: String,
    headers: &'static [(&'static str, &'static str)],
    own: StatusCode,
    foreign: StatusCode,
}

fn route(method: &'static str, uri: &'static str, body: &str, own: StatusCode) -> Route {
    Route {
        method,
        uri,
        body: body.to_owned(),
        headers: &[],
        own,
        foreign: StatusCode::FORBIDDEN,
    }
}

/// The generic DAV routes (PROPFIND, PROPPATCH, DELETE) load the resource before they check
/// the user, so a path that does not exist yet is 404 for everyone.
fn not_yet_there(route: Route) -> Route {
    Route {
        foreign: StatusCode::NOT_FOUND,
        ..route
    }
}

/// Every route, in an order the owner can run them one after another.
fn routes() -> Vec<Route> {
    let depth1: &'static [(&str, &str)] = &[("Depth", "1")];
    let destination: &'static [(&str, &str)] = &[
        ("Destination", "/caldav/principal/user/cal/moved.ics"),
        ("Host", "localhost"),
    ];
    vec![
        Route {
            headers: depth1,
            ..route(
                "PROPFIND",
                "/caldav/principal/user",
                "",
                StatusCode::MULTI_STATUS,
            )
        },
        Route {
            headers: depth1,
            ..route(
                "PROPFIND",
                "/caldav/principal/user/cal",
                "",
                StatusCode::MULTI_STATUS,
            )
        },
        route("GET", "/caldav/principal/user/cal", "", StatusCode::OK),
        route(
            "REPORT",
            "/caldav/principal/user/cal",
            &sync_collection(""),
            StatusCode::MULTI_STATUS,
        ),
        route(
            "REPORT",
            "/caldav/principal/user/cal",
            CALENDAR_QUERY,
            StatusCode::MULTI_STATUS,
        ),
        route(
            "POST",
            "/caldav/principal/user/cal",
            PUSH_REGISTER,
            StatusCode::CREATED,
        ),
        route(
            "PROPPATCH",
            "/caldav/principal/user/cal",
            PROPPATCH_DISPLAYNAME,
            StatusCode::MULTI_STATUS,
        ),
        route(
            "GET",
            "/caldav/principal/user/cal/obj.ics",
            "",
            StatusCode::OK,
        ),
        route(
            "PROPFIND",
            "/caldav/principal/user/cal/obj.ics",
            "",
            StatusCode::MULTI_STATUS,
        ),
        route(
            "PUT",
            "/caldav/principal/user/cal/new.ics",
            &event("new-uid", "New"),
            StatusCode::CREATED,
        ),
        Route {
            headers: destination,
            ..route(
                "MOVE",
                "/caldav/principal/user/cal/new.ics",
                "",
                StatusCode::FORBIDDEN,
            )
        },
        Route {
            headers: destination,
            ..route(
                "COPY",
                "/caldav/principal/user/cal/new.ics",
                "",
                StatusCode::FORBIDDEN,
            )
        },
        not_yet_there(route(
            "DELETE",
            "/caldav/principal/user/cal/new.ics",
            "",
            StatusCode::OK,
        )),
        route(
            "MKCALENDAR",
            "/caldav/principal/user/made",
            MKCALENDAR,
            StatusCode::CREATED,
        ),
        route(
            "IMPORT",
            "/caldav/principal/user/imported",
            &event("imported-uid", "Imported"),
            StatusCode::OK,
        ),
        not_yet_there(route(
            "DELETE",
            "/caldav/principal/user/made",
            "",
            StatusCode::OK,
        )),
        Route {
            headers: depth1,
            ..route(
                "PROPFIND",
                "/carddav/principal/user",
                "",
                StatusCode::MULTI_STATUS,
            )
        },
        Route {
            headers: depth1,
            ..route(
                "PROPFIND",
                "/carddav/principal/user/book",
                "",
                StatusCode::MULTI_STATUS,
            )
        },
        route("GET", "/carddav/principal/user/book", "", StatusCode::OK),
        route(
            "REPORT",
            "/carddav/principal/user/book",
            &sync_collection(""),
            StatusCode::MULTI_STATUS,
        ),
        route(
            "POST",
            "/carddav/principal/user/book",
            PUSH_REGISTER,
            StatusCode::CREATED,
        ),
        route(
            "PROPPATCH",
            "/carddav/principal/user/book",
            PROPPATCH_DISPLAYNAME,
            StatusCode::MULTI_STATUS,
        ),
        route(
            "GET",
            "/carddav/principal/user/book/card.vcf",
            "",
            StatusCode::OK,
        ),
        route(
            "PUT",
            "/carddav/principal/user/book/new.vcf",
            &vcard("new-card", "New"),
            StatusCode::CREATED,
        ),
        not_yet_there(route(
            "DELETE",
            "/carddav/principal/user/book/new.vcf",
            "",
            StatusCode::OK,
        )),
        route(
            "MKCOL",
            "/carddav/principal/user/madebook",
            MKCOL_ADDRESSBOOK,
            StatusCode::CREATED,
        ),
        route(
            "IMPORT",
            "/carddav/principal/user/importedbook",
            &vcard("imported-card", "Imported"),
            StatusCode::OK,
        ),
        not_yet_there(route(
            "DELETE",
            "/carddav/principal/user/madebook",
            "",
            StatusCode::OK,
        )),
    ]
}

#[rstest]
#[tokio::test]
async fn test_route_matrix_own_and_foreign(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let context = context.await;
    add_principal(
        &context,
        "other",
        PrincipalType::Individual,
        Some("otherpass"),
    )
    .await;
    let app = get_app(context.clone());
    seed_user(&app).await;

    // Another authenticated principal: 403 everywhere, nothing changes
    for route in routes() {
        let (status, body) = send(
            &app,
            route.method,
            route.uri,
            Some(OTHER),
            route.headers,
            &route.body,
        )
        .await;
        assert_eq!(
            status, route.foreign,
            "other {} {}: {body}",
            route.method, route.uri
        );
        assert!(
            !body.contains("Secret"),
            "other {} {}",
            route.method,
            route.uri
        );
    }
    // Not authenticated: still 401
    for route in routes() {
        let (status, _) = send(
            &app,
            route.method,
            route.uri,
            None,
            route.headers,
            &route.body,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "anonymous {} {}",
            route.method,
            route.uri
        );
    }
    // The owner: every route works as before
    for route in routes() {
        let (status, body) = send(
            &app,
            route.method,
            route.uri,
            Some(USER),
            route.headers,
            &route.body,
        )
        .await;
        assert_eq!(
            status, route.own,
            "own {} {}: {body}",
            route.method, route.uri
        );
    }
    // The owner still has everything the foreign requests tried to change
    let (status, body) = send(
        &app,
        "GET",
        "/caldav/principal/user/cal/obj.ics",
        Some(USER),
        &[],
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("SUMMARY:Secret"));
}

#[rstest]
#[tokio::test]
async fn test_member_routes_of_a_group(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let context = context.await;
    add_principal(&context, "group.team", PrincipalType::Group, None).await;
    add_principal(
        &context,
        "other",
        PrincipalType::Individual,
        Some("otherpass"),
    )
    .await;
    context
        .principal_store
        .add_membership("user", "group.team")
        .await
        .unwrap();
    let app = get_app(context.clone());

    let (status, _) = send(
        &app,
        "MKCALENDAR",
        "/caldav/principal/group.team/cal",
        Some(USER),
        &[],
        MKCALENDAR,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = send(
        &app,
        "PUT",
        "/caldav/principal/group.team/cal/obj.ics",
        Some(USER),
        &[],
        &event("team-uid", "Team"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = send(
        &app,
        "GET",
        "/caldav/principal/group.team/cal/obj.ics",
        Some(USER),
        &[],
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("SUMMARY:Team"));
    let (status, _) = send(
        &app,
        "GET",
        "/caldav/principal/group.team/cal/obj.ics",
        Some(OTHER),
        &[],
        "",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[rstest]
#[case(
    "/caldav/principal/user/cal",
    "/caldav/principal/user/cal/obj.ics",
    "/caldav/principal/user/cal/new.ics",
    event("new-uid", "New")
)]
#[case(
    "/carddav/principal/user/book",
    "/carddav/principal/user/book/card.vcf",
    "/carddav/principal/user/book/new.vcf",
    vcard("new-card", "New")
)]
#[tokio::test]
async fn test_sync_collection_tokens(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
    #[case] collection: &str,
    #[case] existing: &str,
    #[case] new: &str,
    #[case] new_body: String,
) {
    let context = context.await;
    let app = get_app(context.clone());
    seed_user(&app).await;

    let token_of = |body: &str| {
        let start = body.find("<sync-token>").unwrap() + "<sync-token>".len();
        let end = body[start..].find("</sync-token>").unwrap() + start;
        body[start..end].to_owned()
    };

    let (status, body) = send(
        &app,
        "REPORT",
        collection,
        Some(USER),
        &[],
        &sync_collection(""),
    )
    .await;
    assert_eq!(status, StatusCode::MULTI_STATUS);
    let token = token_of(&body);

    // Delete one object, add another
    let (status, _) = send(&app, "DELETE", existing, Some(USER), &[], "").await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send(&app, "PUT", new, Some(USER), &[], &new_body).await;
    assert_eq!(status, StatusCode::CREATED);

    // The initial sync reports what exists and nothing removed (RFC 6578 section 3.4)
    let (status, body) = send(
        &app,
        "REPORT",
        collection,
        Some(USER),
        &[],
        &sync_collection(""),
    )
    .await;
    assert_eq!(status, StatusCode::MULTI_STATUS);
    assert!(body.contains(&format!("<href>{new}</href>")), "{body}");
    assert!(
        !body.contains(&format!("<href>{existing}</href>")),
        "{body}"
    );
    assert!(!body.contains("404 Not Found"), "{body}");

    // An incremental sync reports the removal
    let (status, body) = send(
        &app,
        "REPORT",
        collection,
        Some(USER),
        &[],
        &sync_collection(&token),
    )
    .await;
    assert_eq!(status, StatusCode::MULTI_STATUS);
    assert!(body.contains(&format!("<href>{existing}</href>")), "{body}");
    assert!(body.contains("404 Not Found"), "{body}");

    // An unknown token is an error (RFC 6578 section 3.2)
    for bad in ["garbage", "github.com/lennart-k/rustical/ns/x"] {
        let (status, body) = send(
            &app,
            "REPORT",
            collection,
            Some(USER),
            &[],
            &sync_collection(bad),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{bad}");
        assert!(body.contains("<valid-sync-token/>"), "{body}");
    }
}

#[rstest]
#[tokio::test]
async fn test_read_only_objects_cannot_be_deleted(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let context = context.await;
    let app = get_app(context.clone());
    let card = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nN:Doe;John;;;,\r\nBDAY:--0203\r\nUID:bday-uid\r\nEND:VCARD\r\n";
    let (status, _) = send(
        &app,
        "IMPORT",
        "/carddav/principal/user/contacts",
        Some(USER),
        &[],
        card,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send(
        &app,
        "MKCOL",
        "/caldav/principal/user/_birthdays_contacts",
        Some(USER),
        &[],
        r#"<mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav"><set><prop>
          <displayname>Birthdays</displayname>
          <CAL:supported-calendar-component-set><CAL:comp name="VEVENT" /></CAL:supported-calendar-component-set>
        </prop></set></mkcol>"#,
    )
    .await;
    assert!(status.is_success(), "{status}");

    let object = "/caldav/principal/user/_birthdays_contacts/bday-uid-birthday.ics";
    let (status, _) = send(&app, "GET", object, Some(USER), &[], "").await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send(&app, "DELETE", object, Some(USER), &[], "").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = send(&app, "GET", object, Some(USER), &[], "").await;
    assert_eq!(status, StatusCode::OK);
}

#[rstest]
#[tokio::test]
async fn test_collections_cannot_take_a_share_name(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    use rustical_store::{Access, ShareStore};
    let context = context.await;
    add_principal(
        &context,
        "other",
        PrincipalType::Individual,
        Some("otherpass"),
    )
    .await;
    let app = get_app(context.clone());
    seed_user(&app).await;
    let calendar = context
        .share_store
        .share_calendar("user", "cal", "other", Access::Read, None)
        .await
        .unwrap();
    let addressbook = context
        .share_store
        .share_addressbook("user", "book", "other", Access::Read, None)
        .await
        .unwrap();

    // As for any existing collection: MKCALENDAR/MKCOL 405 (RFC 4791 section 5.3.1), IMPORT 409
    for (method, uri, body, expected) in [
        (
            "MKCALENDAR",
            format!("/caldav/principal/other/{}", calendar.name),
            MKCALENDAR.to_owned(),
            StatusCode::METHOD_NOT_ALLOWED,
        ),
        (
            "IMPORT",
            format!("/caldav/principal/other/{}", calendar.name),
            event("imported-uid", "Imported"),
            StatusCode::CONFLICT,
        ),
        (
            "MKCOL",
            format!("/carddav/principal/other/{}", addressbook.name),
            MKCOL_ADDRESSBOOK.to_owned(),
            StatusCode::METHOD_NOT_ALLOWED,
        ),
        (
            "IMPORT",
            format!("/carddav/principal/other/{}", addressbook.name),
            vcard("imported-card", "Imported"),
            StatusCode::CONFLICT,
        ),
    ] {
        let (status, body) = send(&app, method, &uri, Some(OTHER), &[], &body).await;
        assert_eq!(status, expected, "{method} {uri}: {body}");
    }
}
