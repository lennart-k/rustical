//! Shared calendars and addressbooks over CalDAV and CardDAV.
//!
//! `user` owns `cal` and `book` and shares `cal` read-only with `bob` and read-write with the
//! group `group.team` (member: `erin`, not `user`), `book` read-write with `bob`. `other` has
//! no access.
use super::access::{
    MKCALENDAR, MKCOL_ADDRESSBOOK, OTHER, PROPPATCH_DISPLAYNAME, PUSH_REGISTER, USER,
    add_principal, event, sync_collection, vcard,
};
use super::get_app;
use super::visibility::canary_event;
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::{HeaderMap, StatusCode};
use rstest::rstest;
use rustical_store::auth::{AuthenticationProvider, PrincipalType};
use rustical_store::shared::Policy;
use rustical_store::{Access, ShareStore};
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
use tower::ServiceExt;

const BOB: (&str, &str) = ("bob", "bobpass");
const ERIN: (&str, &str) = ("erin", "erinpass");
/// The group home of erin (Apple clients reach it as `erin$group.team`)
const ERIN_AS_TEAM: (&str, &str) = ("erin$group.team", "erinpass");

const BOB_CAL: &str = "/caldav/principal/bob/cal_shared_by_user";
const TEAM_CAL: &str = "/caldav/principal/group.team/cal_shared_by_user";
const BOB_BOOK: &str = "/carddav/principal/bob/book_shared_by_user";
const OWN_CAL: &str = "/caldav/principal/user/cal";

const ALARM: &str =
    "BEGIN:VALARM\r\nACTION:DISPLAY\r\nDESCRIPTION:Owner alarm\r\nTRIGGER:-PT15M\r\nEND:VALARM\r\n";

/// An event with extra lines inside its VEVENT
fn event_with(uid: &str, lines: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\nUID:{uid}\r\n\
         DTSTAMP:20260101T000000Z\r\nDTSTART:20260105T100000Z\r\nDTEND:20260105T110000Z\r\n\
         {lines}END:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

struct Answer {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

async fn call(
    app: &axum::Router,
    method: &str,
    uri: &str,
    auth: Option<(&str, &str)>,
    headers: &[(&str, &str)],
    body: &str,
) -> Answer {
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
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    Answer {
        status,
        headers,
        body: String::from_utf8(bytes.to_vec()).unwrap(),
    }
}

async fn ok(app: &axum::Router, method: &str, uri: &str, auth: (&str, &str), body: &str) -> Answer {
    let answer = call(app, method, uri, Some(auth), &[], body).await;
    assert!(
        answer.status.is_success(),
        "{method} {uri} as {}: {} {}",
        auth.0,
        answer.status,
        answer.body
    );
    answer
}

fn etag(answer: &Answer) -> String {
    answer
        .headers
        .get("ETag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

fn sync_token(body: &str) -> String {
    let start = body.find("<sync-token>").unwrap() + "<sync-token>".len();
    let end = body[start..].find("</sync-token>").unwrap() + start;
    body[start..end].to_owned()
}

async fn setup() -> (TestStoreContext, axum::Router) {
    let context = test_store_context().await;
    add_principal(&context, "bob", PrincipalType::Individual, Some("bobpass")).await;
    add_principal(
        &context,
        "erin",
        PrincipalType::Individual,
        Some("erinpass"),
    )
    .await;
    add_principal(
        &context,
        "other",
        PrincipalType::Individual,
        Some("otherpass"),
    )
    .await;
    add_principal(&context, "group.team", PrincipalType::Group, None).await;
    context
        .principal_store
        .add_membership("erin", "group.team")
        .await
        .unwrap();
    let app = get_app(context.clone());

    ok(&app, "MKCALENDAR", OWN_CAL, USER, MKCALENDAR).await;
    for (id, lines) in [
        ("public", format!("SUMMARY:Lunch\r\n{ALARM}")),
        ("private", "CLASS:PRIVATE\r\nSUMMARY:Doctor\r\n".to_owned()),
        (
            "confidential",
            "CLASS:CONFIDENTIAL\r\nSUMMARY:Interview\r\n".to_owned(),
        ),
    ] {
        ok(
            &app,
            "PUT",
            &format!("{OWN_CAL}/{id}.ics"),
            USER,
            &event_with(&format!("{id}-uid"), &lines),
        )
        .await;
    }
    ok(
        &app,
        "MKCOL",
        "/carddav/principal/user/book",
        USER,
        MKCOL_ADDRESSBOOK,
    )
    .await;
    ok(
        &app,
        "PUT",
        "/carddav/principal/user/book/card.vcf",
        USER,
        &vcard("card-uid", "Card"),
    )
    .await;

    let shares = &context.share_store;
    shares
        .share_calendar("user", "cal", "bob", Access::Read, Some("user"))
        .await
        .unwrap();
    shares
        .share_calendar("user", "cal", "group.team", Access::Write, Some("user"))
        .await
        .unwrap();
    shares
        .share_addressbook("user", "book", "bob", Access::Write, Some("user"))
        .await
        .unwrap();
    (context, app)
}

const DEPTH1: &[(&str, &str)] = &[("Depth", "1")];

#[rstest]
#[tokio::test]
async fn test_sharees_see_the_shared_collections() {
    let (_context, app) = setup().await;

    // in bob's home, with the owner and read-only privileges
    let home = call(
        &app,
        "PROPFIND",
        "/caldav/principal/bob",
        Some(BOB),
        DEPTH1,
        "",
    )
    .await;
    assert_eq!(home.status, StatusCode::MULTI_STATUS);
    assert!(
        home.body
            .contains("<href>/caldav/principal/bob/cal_shared_by_user/</href>"),
        "{}",
        home.body
    );
    let shared = call(&app, "PROPFIND", BOB_CAL, Some(BOB), &[("Depth", "0")], "").await;
    assert_eq!(shared.status, StatusCode::MULTI_STATUS);
    assert!(
        shared
            .body
            .contains("<owner>\n                    <href>/caldav/principal/user/</href>"),
        "{}",
        shared.body
    );
    assert!(
        !shared.body.contains("<write/>") && !shared.body.contains("<all/>"),
        "{}",
        shared.body
    );
    assert!(
        shared.body.contains("<write-properties/>"),
        "{}",
        shared.body
    );

    // The group's share is in the group home, with write access
    for auth in [ERIN, ERIN_AS_TEAM] {
        let home = call(
            &app,
            "PROPFIND",
            "/caldav/principal/group.team",
            Some(auth),
            DEPTH1,
            "",
        )
        .await;
        assert_eq!(home.status, StatusCode::MULTI_STATUS, "{}", auth.0);
        assert!(home.body.contains("cal_shared_by_user"), "{}", home.body);
    }
    let shared = call(
        &app,
        "PROPFIND",
        TEAM_CAL,
        Some(ERIN),
        &[("Depth", "0")],
        "",
    )
    .await;
    assert!(shared.body.contains("<write/>"), "{}", shared.body);

    // /caldav-compat serves the user's shares too
    let compat = call(
        &app,
        "PROPFIND",
        "/caldav-compat/principal/bob",
        Some(BOB),
        DEPTH1,
        "",
    )
    .await;
    assert_eq!(compat.status, StatusCode::MULTI_STATUS);
    assert!(
        compat.body.contains("cal_shared_by_user"),
        "{}",
        compat.body
    );

    // Addressbooks
    let home = call(
        &app,
        "PROPFIND",
        "/carddav/principal/bob",
        Some(BOB),
        DEPTH1,
        "",
    )
    .await;
    assert!(home.body.contains("book_shared_by_user"), "{}", home.body);
    let card = ok(&app, "GET", &format!("{BOB_BOOK}/card.vcf"), BOB, "").await;
    assert!(card.body.contains("FN:Card"));

    // Not the shares of others (the owner's principal shows its name only:
    // test_sharees_see_who_shares)
    for (uri, auth) in [(BOB_CAL, OTHER), (TEAM_CAL, BOB), (BOB_BOOK, OTHER)] {
        let answer = call(&app, "PROPFIND", uri, Some(auth), &[], "").await;
        assert_eq!(answer.status, StatusCode::FORBIDDEN, "{uri} as {}", auth.0);
    }
}

#[rstest]
#[tokio::test]
async fn test_reads_through_a_share_show_views() {
    let (context, app) = setup().await;

    // Without a setting of the calendar PRIVATE is hidden too
    let private = call(
        &app,
        "GET",
        &format!("{BOB_CAL}/private.ics"),
        Some(BOB),
        &[],
        "",
    )
    .await;
    assert_eq!(private.status, StatusCode::NOT_FOUND);
    context
        .share_store
        .set_visibility("user", "cal", Policy::new(1, 0).unwrap())
        .await
        .unwrap();

    // PUBLIC in full but without alarms; PRIVATE busy only; CONFIDENTIAL hidden
    let public = ok(&app, "GET", &format!("{BOB_CAL}/public.ics"), BOB, "").await;
    assert!(public.body.contains("SUMMARY:Lunch"));
    assert!(!public.body.contains("VALARM"));
    let private = ok(&app, "GET", &format!("{BOB_CAL}/private.ics"), BOB, "").await;
    assert!(!private.body.contains("Doctor"));
    assert!(private.body.contains("DTSTART:20260105T100000Z"));
    let hidden = call(
        &app,
        "GET",
        &format!("{BOB_CAL}/confidential.ics"),
        Some(BOB),
        &[],
        "",
    )
    .await;
    assert_eq!(hidden.status, StatusCode::NOT_FOUND);

    // The same for listings, exports and reports
    let members = call(&app, "PROPFIND", BOB_CAL, Some(BOB), DEPTH1, "").await;
    assert!(
        !members.body.contains("confidential.ics"),
        "{}",
        members.body
    );
    assert!(members.body.contains("private.ics"));
    let export = ok(&app, "GET", BOB_CAL, BOB, "").await;
    assert!(!export.body.contains("Interview") && !export.body.contains("Doctor"));
    assert!(!export.body.contains("VALARM"));
    let sync = call(
        &app,
        "REPORT",
        BOB_CAL,
        Some(BOB),
        &[],
        &sync_collection(""),
    )
    .await;
    assert!(!sync.body.contains("confidential.ics"), "{}", sync.body);

    // The owner's view is unchanged
    let own = ok(
        &app,
        "GET",
        &format!("{OWN_CAL}/confidential.ics"),
        USER,
        "",
    )
    .await;
    assert!(own.body.contains("Interview"));
}

const MARKER_QUERY: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<C:calendar-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:prop><D:getetag/></D:prop>
  <C:filter><C:comp-filter name="VCALENDAR"><C:comp-filter name="VEVENT">
    <C:prop-filter name="{prop}"><C:text-match collation="i;ascii-casemap">{text}</C:text-match></C:prop-filter>
  </C:comp-filter></C:comp-filter></C:filter>
</C:calendar-query>"#;

fn marker_query(prop: &str, text: &str) -> String {
    MARKER_QUERY.replace("{prop}", prop).replace("{text}", text)
}

const ALL_QUERY: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<C:calendar-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:prop><D:getetag/><C:calendar-data/></D:prop>
  <C:filter><C:comp-filter name="VCALENDAR"/></C:filter>
</C:calendar-query>"#;

fn multiget(href: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8" ?>
<C:calendar-multiget xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:prop><D:getetag/><C:calendar-data/></D:prop>
  <D:href>{href}</D:href>
</C:calendar-multiget>"#
    )
}

/// Every endpoint a sharee can read the canary through
async fn canary_bodies(app: &axum::Router, token: &str) -> Vec<(String, String)> {
    let object = format!("{BOB_CAL}/canary.ics");
    let mut bodies = vec![];
    for (name, method, uri, headers, body) in [
        ("GET object", "GET", object.as_str(), &[][..], String::new()),
        ("export", "GET", BOB_CAL, &[][..], String::new()),
        ("PROPFIND", "PROPFIND", BOB_CAL, DEPTH1, String::new()),
        (
            "PROPFIND object",
            "PROPFIND",
            object.as_str(),
            &[][..],
            String::new(),
        ),
        ("multiget", "REPORT", BOB_CAL, &[][..], multiget(&object)),
        ("query", "REPORT", BOB_CAL, &[][..], ALL_QUERY.to_owned()),
        (
            "initial sync",
            "REPORT",
            BOB_CAL,
            &[][..],
            sync_collection(""),
        ),
        (
            "incremental sync",
            "REPORT",
            BOB_CAL,
            &[][..],
            sync_collection(token),
        ),
    ] {
        let answer = call(app, method, uri, Some(BOB), headers, &body).await;
        assert!(
            answer.status.is_success() || answer.status == StatusCode::NOT_FOUND,
            "{name}: {}",
            answer.status
        );
        bodies.push((name.to_owned(), answer.body));
    }
    bodies
}

#[rstest]
#[tokio::test]
async fn test_canary_never_leaks() {
    let (context, app) = setup().await;
    let start = call(
        &app,
        "REPORT",
        BOB_CAL,
        Some(BOB),
        &[],
        &sync_collection(""),
    )
    .await;
    let token = sync_token(&start.body);
    ok(
        &app,
        "PUT",
        &format!("{OWN_CAL}/canary.ics"),
        USER,
        &canary_event("CLASS:PRIVATE\r\n"),
    )
    .await;

    // Busy (the default for PRIVATE): no marker anywhere
    for (name, body) in canary_bodies(&app, &token).await {
        assert!(!body.contains("MARKER-"), "{name}: {body}");
    }
    // A text match on a hidden field finds nothing: no yes/no oracle
    for (prop, text) in [
        ("SUMMARY", "MARKER-SUMMARY"),
        ("DESCRIPTION", "MARKER-DESCRIPTION"),
        ("LOCATION", "MARKER"),
    ] {
        let answer = call(
            &app,
            "REPORT",
            BOB_CAL,
            Some(BOB),
            &[],
            &marker_query(prop, text),
        )
        .await;
        assert!(
            !answer.body.contains("canary.ics"),
            "{prop}: {}",
            answer.body
        );
    }
    let owner = call(
        &app,
        "REPORT",
        OWN_CAL,
        Some(USER),
        &[],
        &marker_query("DESCRIPTION", "MARKER-DESCRIPTION"),
    )
    .await;
    assert!(owner.body.contains("canary.ics"), "{}", owner.body);

    // Title and location: exactly those two markers
    context
        .share_store
        .set_visibility("user", "cal", Policy::new(3, 0).unwrap())
        .await
        .unwrap();
    for (name, body) in canary_bodies(&app, &token).await {
        for marker in body
            .match_indices("MARKER-")
            .map(|(i, _)| &body[i..(i + 20).min(body.len())])
        {
            assert!(
                marker.starts_with("MARKER-SUMMARY") || marker.starts_with("MARKER-LOCATION"),
                "{name}: {marker}"
            );
        }
    }
    let answer = call(
        &app,
        "REPORT",
        BOB_CAL,
        Some(BOB),
        &[],
        &marker_query("SUMMARY", "MARKER-SUMMARY"),
    )
    .await;
    assert!(answer.body.contains("canary.ics"), "{}", answer.body);
    let answer = call(
        &app,
        "REPORT",
        BOB_CAL,
        Some(BOB),
        &[],
        &marker_query("DESCRIPTION", "MARKER-DESCRIPTION"),
    )
    .await;
    assert!(!answer.body.contains("canary.ics"), "{}", answer.body);

    // Hidden: only as a removed member of an incremental sync
    context
        .share_store
        .set_visibility("user", "cal", Policy::new(0, 0).unwrap())
        .await
        .unwrap();
    for (name, body) in canary_bodies(&app, &token).await {
        assert!(!body.contains("MARKER-"), "{name}: {body}");
        // The removed member of an incremental sync; the multiget answers the href it asked for
        if name == "incremental sync" || name == "multiget" {
            assert!(
                body.contains("canary.ics") && body.contains("404 Not Found"),
                "{body}"
            );
        } else {
            assert!(!body.contains("canary.ics"), "{name}: {body}");
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_writes_through_a_share() {
    let (context, app) = setup().await;

    // The write share of the group: change a public event with the ETag of its view
    let seen = ok(&app, "GET", &format!("{TEAM_CAL}/public.ics"), ERIN, "").await;
    let answer = call(
        &app,
        "PUT",
        &format!("{TEAM_CAL}/public.ics"),
        Some(ERIN),
        &[("If-Match", &etag(&seen))],
        &event_with("public-uid", "SUMMARY:Lunch moved\r\n"),
    )
    .await;
    assert_eq!(answer.status, StatusCode::CREATED, "{}", answer.body);
    // The owner's alarm survives
    let own = ok(&app, "GET", &format!("{OWN_CAL}/public.ics"), USER, "").await;
    assert!(
        own.body.contains("Lunch moved") && own.body.contains("Owner alarm"),
        "{}",
        own.body
    );
    // The old ETag no longer matches
    let answer = call(
        &app,
        "PUT",
        &format!("{TEAM_CAL}/public.ics"),
        Some(ERIN),
        &[("If-Match", &etag(&seen))],
        &event_with("public-uid", "SUMMARY:Again\r\n"),
    )
    .await;
    assert_eq!(answer.status, StatusCode::PRECONDITION_FAILED);

    // New objects
    ok(
        &app,
        "PUT",
        &format!("{TEAM_CAL}/new.ics"),
        ERIN,
        &event_with("new-uid", "SUMMARY:New\r\n"),
    )
    .await;
    assert!(
        ok(&app, "GET", &format!("{OWN_CAL}/new.ics"), USER, "")
            .await
            .body
            .contains("SUMMARY:New")
    );

    // What a sharee may not do
    let answer = call(
        &app,
        "PUT",
        &format!("{TEAM_CAL}/private.ics"),
        Some(ERIN),
        &[],
        &event_with("private-uid", ""),
    )
    .await;
    assert_eq!(answer.status, StatusCode::FORBIDDEN, "not fully visible");
    let answer = call(
        &app,
        "PUT",
        &format!("{TEAM_CAL}/other.ics"),
        Some(ERIN),
        &[],
        &event_with("public-uid", ""),
    )
    .await;
    assert_eq!(answer.status, StatusCode::CONFLICT, "foreign UID");
    assert!(answer.body.contains("no-uid-conflict"), "{}", answer.body);
    let answer = call(
        &app,
        "PUT",
        &format!("{TEAM_CAL}/public.ics"),
        Some(ERIN),
        &[],
        &event_with("changed-uid", ""),
    )
    .await;
    assert_eq!(answer.status, StatusCode::CONFLICT, "changed UID");
    ok(&app, "DELETE", &format!("{OWN_CAL}/new.ics"), USER, "").await;
    let answer = call(
        &app,
        "PUT",
        &format!("{TEAM_CAL}/new.ics"),
        Some(ERIN),
        &[],
        &event_with("new-uid", ""),
    )
    .await;
    assert_eq!(
        answer.status,
        StatusCode::CONFLICT,
        "name in the owner's trash bin"
    );
    let answer = call(
        &app,
        "PUT",
        &format!("{BOB_CAL}/bob.ics"),
        Some(BOB),
        &[],
        &event_with("bob-uid", ""),
    )
    .await;
    assert_eq!(answer.status, StatusCode::FORBIDDEN, "read-only share");
    let answer = call(
        &app,
        "DELETE",
        &format!("{BOB_CAL}/public.ics"),
        Some(BOB),
        &[],
        "",
    )
    .await;
    assert_eq!(answer.status, StatusCode::FORBIDDEN, "read-only share");

    // A delete through the share checks the ETag of the view (the owner's alarm does not count)
    // and goes to the owner's trash bin, also when asked for a hard one
    let seen = ok(&app, "GET", &format!("{TEAM_CAL}/public.ics"), ERIN, "").await;
    let answer = call(
        &app,
        "DELETE",
        &format!("{TEAM_CAL}/public.ics"),
        Some(ERIN),
        &[("If-Match", "\"stale\"")],
        "",
    )
    .await;
    assert_eq!(answer.status, StatusCode::PRECONDITION_FAILED);
    let answer = call(
        &app,
        "DELETE",
        &format!("{TEAM_CAL}/public.ics"),
        Some(ERIN),
        &[("X-No-Trashbin", "1"), ("If-Match", &etag(&seen))],
        "",
    )
    .await;
    assert!(answer.status.is_success(), "{}", answer.status);
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("{OWN_CAL}/public.ics"),
            Some(USER),
            &[],
            ""
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    use rustical_store::CalendarReadStore;
    assert!(
        context
            .cal_store
            .get_object("user", "cal", "public", true)
            .await
            .is_ok()
    );

    // a sharee may make an object invisible to itself
    let start = call(
        &app,
        "REPORT",
        TEAM_CAL,
        Some(ERIN),
        &[],
        &sync_collection(""),
    )
    .await;
    let token = sync_token(&start.body);
    ok(
        &app,
        "PUT",
        &format!("{TEAM_CAL}/mine.ics"),
        ERIN,
        &event_with("mine-uid", "SUMMARY:Mine\r\n"),
    )
    .await;
    let answer = call(
        &app,
        "PUT",
        &format!("{TEAM_CAL}/mine.ics"),
        Some(ERIN),
        &[],
        &event_with("mine-uid", "CLASS:CONFIDENTIAL\r\nSUMMARY:Mine\r\n"),
    )
    .await;
    assert_eq!(answer.status, StatusCode::CREATED);
    assert!(
        answer.headers.get("ETag").is_none(),
        "nothing visible to tag"
    );
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("{TEAM_CAL}/mine.ics"),
            Some(ERIN),
            &[],
            ""
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let sync = call(
        &app,
        "REPORT",
        TEAM_CAL,
        Some(ERIN),
        &[],
        &sync_collection(&token),
    )
    .await;
    assert!(
        sync.body.contains("mine.ics") && sync.body.contains("404 Not Found"),
        "{}",
        sync.body
    );

    // Contacts
    ok(
        &app,
        "PUT",
        &format!("{BOB_BOOK}/bob.vcf"),
        BOB,
        &vcard("bob-card", "Bob's"),
    )
    .await;
    assert!(
        ok(
            &app,
            "GET",
            "/carddav/principal/user/book/bob.vcf",
            USER,
            ""
        )
        .await
        .body
        .contains("Bob's")
    );
    ok(
        &app,
        "DELETE",
        "/carddav/principal/user/book/bob.vcf",
        USER,
        "",
    )
    .await;
    let answer = call(
        &app,
        "PUT",
        &format!("{BOB_BOOK}/bob.vcf"),
        Some(BOB),
        &[],
        &vcard("bob-card", "Again"),
    )
    .await;
    assert_eq!(
        answer.status,
        StatusCode::CONFLICT,
        "card in the owner's trash bin"
    );
}

#[rstest]
#[tokio::test]
async fn test_sharee_view_properties() {
    let (_context, app) = setup().await;
    // name and color for the sharee only
    let answer = ok(
        &app,
        "PROPPATCH",
        BOB_CAL,
        BOB,
        r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propertyupdate xmlns:D="DAV:" xmlns:I="http://apple.com/ns/ical/">
  <D:set><D:prop><D:displayname>Bobs view</D:displayname><I:calendar-color>#ff0000</I:calendar-color></D:prop></D:set>
</D:propertyupdate>"#,
    )
    .await;
    assert!(!answer.body.contains("409"), "{}", answer.body);
    let shared = call(&app, "PROPFIND", BOB_CAL, Some(BOB), &[], "").await;
    assert!(
        shared.body.contains("Bobs view") && shared.body.contains("#ff0000"),
        "{}",
        shared.body
    );
    let own = call(&app, "PROPFIND", OWN_CAL, Some(USER), &[], "").await;
    assert!(
        !own.body.contains("Bobs view") && !own.body.contains("#ff0000"),
        "{}",
        own.body
    );
    let team = call(&app, "PROPFIND", TEAM_CAL, Some(ERIN), &[], "").await;
    assert!(!team.body.contains("Bobs view"), "{}", team.body);

    // Anything else: the whole request is refused
    let answer = call(
        &app,
        "PROPPATCH",
        BOB_CAL,
        Some(BOB),
        &[],
        r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propertyupdate xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:set><D:prop><D:displayname>Changed</D:displayname><C:calendar-description>Mine</C:calendar-description></D:prop></D:set>
</D:propertyupdate>"#,
    )
    .await;
    assert!(answer.body.contains("409"), "{}", answer.body);
    let shared = call(&app, "PROPFIND", BOB_CAL, Some(BOB), &[], "").await;
    assert!(
        shared.body.contains("Bobs view") && !shared.body.contains("Changed"),
        "{}",
        shared.body
    );

    let answer = ok(&app, "PROPPATCH", BOB_BOOK, BOB, PROPPATCH_DISPLAYNAME).await;
    assert!(!answer.body.contains("409"), "{}", answer.body);
    let own = call(
        &app,
        "PROPFIND",
        "/carddav/principal/user/book",
        Some(USER),
        &[],
        "",
    )
    .await;
    assert!(!own.body.contains("Renamed"), "{}", own.body);
}

#[rstest]
#[tokio::test]
async fn test_share_lifecycle() {
    let (context, app) = setup().await;
    let shares = &context.share_store;

    // Access changes act at once
    let answer = call(
        &app,
        "PUT",
        &format!("{BOB_CAL}/bob.ics"),
        Some(BOB),
        &[],
        &event_with("bob-uid", ""),
    )
    .await;
    assert_eq!(answer.status, StatusCode::FORBIDDEN);
    shares
        .share_calendar("user", "cal", "bob", Access::Write, None)
        .await
        .unwrap();
    ok(
        &app,
        "PUT",
        &format!("{BOB_CAL}/bob.ics"),
        BOB,
        &event_with("bob-uid", ""),
    )
    .await;

    // Trash bin: gone for the sharee (the home stays listable), back after restore
    use rustical_store::{CalendarReadStore, CalendarWriteStore};
    context
        .cal_store
        .delete_calendar("user", "cal", true)
        .await
        .unwrap();
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("{BOB_CAL}/bob.ics"),
            Some(BOB),
            &[],
            ""
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let home = call(
        &app,
        "PROPFIND",
        "/caldav/principal/bob",
        Some(BOB),
        DEPTH1,
        "",
    )
    .await;
    assert_eq!(home.status, StatusCode::MULTI_STATUS);
    assert!(!home.body.contains("cal_shared_by_user"), "{}", home.body);
    context
        .cal_store
        .restore_calendar("user", "cal")
        .await
        .unwrap();
    ok(&app, "GET", &format!("{BOB_CAL}/bob.ics"), BOB, "").await;

    // the sharee removes the share; the owner keeps the calendar
    ok(&app, "DELETE", BOB_CAL, BOB, "").await;
    assert_eq!(
        call(&app, "PROPFIND", BOB_CAL, Some(BOB), &[], "")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    // the owner sees who it is shared with
    let listed: Vec<_> = shares
        .list_calendar_shares("user", "cal")
        .await
        .unwrap()
        .into_iter()
        .map(|share| share.sharee)
        .collect();
    assert_eq!(listed, ["group.team"]);
    assert!(
        context
            .cal_store
            .get_calendar("user", "cal", false)
            .await
            .is_ok()
    );

    // the owner ends a share
    shares
        .unshare_calendar("user", "cal", "group.team")
        .await
        .unwrap();
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("{TEAM_CAL}/private.ics"),
            Some(ERIN),
            &[],
            ""
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    // Purge removes every share of the calendar
    shares
        .share_calendar("user", "cal", "bob", Access::Read, None)
        .await
        .unwrap();
    context
        .cal_store
        .delete_calendar("user", "cal", false)
        .await
        .unwrap();
    assert!(
        shares
            .list_calendar_shares("user", "cal")
            .await
            .unwrap()
            .is_empty()
    );

    // Addressbooks
    ok(&app, "DELETE", BOB_BOOK, BOB, "").await;
    assert!(
        shares
            .list_addressbook_shares("user", "book")
            .await
            .unwrap()
            .is_empty()
    );
    ok(
        &app,
        "GET",
        "/carddav/principal/user/book/card.vcf",
        USER,
        "",
    )
    .await;
}

#[rstest]
#[tokio::test]
async fn test_groups_own_and_receive_shares() {
    let (context, app) = setup().await;
    ok(
        &app,
        "MKCALENDAR",
        "/caldav/principal/group.team/teamcal",
        ERIN,
        MKCALENDAR,
    )
    .await;
    ok(
        &app,
        "PUT",
        "/caldav/principal/group.team/teamcal/t.ics",
        ERIN,
        &event_with("t-uid", "SUMMARY:Team\r\n"),
    )
    .await;
    context
        .share_store
        .share_calendar("group.team", "teamcal", "bob", Access::Read, Some("erin"))
        .await
        .unwrap();
    let answer = ok(
        &app,
        "GET",
        "/caldav/principal/bob/teamcal_shared_by_group.team/t.ics",
        BOB,
        "",
    )
    .await;
    assert!(answer.body.contains("SUMMARY:Team"));
    let answer = call(
        &app,
        "PROPFIND",
        "/caldav/principal/bob/teamcal_shared_by_group.team",
        Some(BOB),
        &[],
        "",
    )
    .await;
    assert!(
        answer
            .body
            .contains("<href>/caldav/principal/group.team/</href>"),
        "{}",
        answer.body
    );
    // erin left the group: no more access to the group home and its shares
    context
        .principal_store
        .remove_membership("erin", "group.team")
        .await
        .unwrap();
    assert_eq!(
        call(&app, "GET", TEAM_CAL, Some(ERIN), &[], "")
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

#[rstest]
#[tokio::test]
async fn test_push_subscriptions_expire_within_a_week() {
    let (_context, app) = setup().await;
    let far = PUSH_REGISTER.replace(
        "</subscription>",
        "</subscription><expires>Fri, 01 Jan 2100 00:00:00 GMT</expires>",
    );
    for (uri, auth) in [(OWN_CAL, USER), (BOB_CAL, BOB), (BOB_BOOK, BOB)] {
        let answer = ok(&app, "POST", uri, auth, &far).await;
        let expires = answer.headers.get("Expires").unwrap().to_str().unwrap();
        let expires = chrono::DateTime::parse_from_rfc2822(expires).unwrap();
        assert!(
            expires
                <= chrono::Utc::now() + chrono::Duration::days(7) + chrono::Duration::minutes(1),
            "{uri}: {expires}"
        );
    }
    assert_eq!(
        call(&app, "POST", BOB_CAL, Some(OTHER), &[], PUSH_REGISTER)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

/// Every route with an own collection, a write share, a read share and as another principal
#[rstest]
#[tokio::test]
async fn test_route_matrix_with_shares() {
    let (_context, app) = setup().await;
    let routes = |cal: &str, kind: &str| {
        vec![
            ("PROPFIND", cal.to_owned(), String::new()),
            ("GET", cal.to_owned(), String::new()),
            ("REPORT", cal.to_owned(), sync_collection("")),
            ("POST", cal.to_owned(), PUSH_REGISTER.to_owned()),
            ("GET", format!("{cal}/public.ics"), String::new()),
            ("PROPFIND", format!("{cal}/public.ics"), String::new()),
            (
                "PUT",
                format!("{cal}/m-{kind}.ics"),
                event_with(&format!("m-{kind}"), ""),
            ),
            // Who may not write tries an existing object (a missing one is 404 for everyone)
            (
                "DELETE",
                if matches!(kind, "read" | "other") {
                    format!("{cal}/public.ics")
                } else {
                    format!("{cal}/m-{kind}.ics")
                },
                String::new(),
            ),
            ("MKCALENDAR", cal.to_owned(), MKCALENDAR.to_owned()),
            ("IMPORT", cal.to_owned(), event("i-uid", "Imported")),
        ]
    };
    let expect = |kind: &str, method: &str| -> Vec<StatusCode> {
        use StatusCode as S;
        match (kind, method) {
            ("other", _) => vec![S::FORBIDDEN],
            (_, "PROPFIND" | "REPORT") => vec![S::MULTI_STATUS],
            (_, "GET") => vec![S::OK],
            (_, "POST") => vec![S::CREATED],
            ("read", "PUT" | "DELETE") => vec![S::FORBIDDEN],
            (_, "PUT") => vec![S::CREATED],
            (_, "DELETE") => vec![S::OK],
            // The name is taken
            (_, "MKCALENDAR") => vec![S::METHOD_NOT_ALLOWED],
            // The owner merges into its calendar
            ("own", "IMPORT") => vec![S::OK],
            (_, "IMPORT") => vec![S::CONFLICT],
            _ => unreachable!(),
        }
    };
    for (kind, cal, auth) in [
        ("own", OWN_CAL, USER),
        ("write", TEAM_CAL, ERIN),
        ("read", BOB_CAL, BOB),
        ("other", BOB_CAL, OTHER),
        ("other", OWN_CAL, BOB),
    ] {
        for (method, uri, body) in routes(cal, kind) {
            let answer = call(
                &app,
                method,
                &uri,
                Some(auth),
                &[("Host", "localhost")],
                &body,
            )
            .await;
            assert!(
                expect(kind, method).contains(&answer.status),
                "{kind} {method} {uri}: {} {}",
                answer.status,
                answer.body
            );
        }
        for method in ["MOVE", "COPY"] {
            let answer = call(
                &app,
                method,
                &format!("{cal}/private.ics"),
                Some(auth),
                &[
                    ("Destination", &format!("{cal}/x.ics")),
                    ("Host", "localhost"),
                ],
                "",
            )
            .await;
            assert_eq!(answer.status, StatusCode::FORBIDDEN, "{kind} {method}");
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_oidc_login_keeps_the_email_current() {
    use rustical_frontend::OidcUserStore;
    use rustical_oidc::UserStore;
    let (context, _app) = setup().await;
    let store = OidcUserStore(std::sync::Arc::new(context.principal_store.clone()));
    let shares = &context.share_store;

    // Writes the claim
    store
        .ensure_user("dave", &[], Some("Dave@Example.org"))
        .await
        .unwrap();
    assert_eq!(
        shares
            .resolve_email("dave@example.org")
            .await
            .unwrap()
            .as_deref(),
        Some("dave")
    );
    // Keeps it without a claim
    store.ensure_user("dave", &[], None).await.unwrap();
    assert_eq!(
        shares.principal_email("dave").await.unwrap().as_deref(),
        Some("dave@example.org")
    );
    // Moves it to the newest claim
    store
        .ensure_user("bob", &[], Some("dave@example.org"))
        .await
        .unwrap();
    assert_eq!(
        shares
            .resolve_email("dave@example.org")
            .await
            .unwrap()
            .as_deref(),
        Some("bob")
    );
    assert_eq!(shares.principal_email("dave").await.unwrap(), None);
    // shares by e-mail address
    let share = shares
        .share_calendar("user", "cal", "DAVE@example.org", Access::Read, None)
        .await;
    assert_eq!(share.unwrap().sharee, "bob");
    let _ = context
        .principal_store
        .get_principal("dave")
        .await
        .unwrap()
        .unwrap();
}

mod frontend {
    use super::*;

    const SAME_SITE: &[(&str, &str)] = &[
        ("Host", "localhost"),
        ("Origin", "http://localhost"),
        ("Content-Type", "application/x-www-form-urlencoded"),
    ];
    const SHARES: &str = "/frontend/user/user/calendar/cal/shares";

    #[rstest]
    #[tokio::test]
    async fn test_posts_need_this_origin() {
        let (context, app) = setup().await;
        for headers in [
            &[("Host", "localhost"), ("Origin", "http://evil.example")][..],
            &[
                ("Host", "localhost"),
                ("Referer", "http://evil.example/page"),
            ][..],
            &[("Host", "localhost")][..],
        ] {
            let answer = call(
                &app,
                "POST",
                &format!("{SHARES}/add"),
                Some(USER),
                headers,
                "sharee=other&access=write",
            )
            .await;
            assert_eq!(answer.status, StatusCode::FORBIDDEN, "{headers:?}");
        }
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/visibility"),
            Some(USER),
            &[("Host", "localhost"), ("Origin", "http://evil.example")],
            "private=4&confidential=4",
        )
        .await;
        assert_eq!(answer.status, StatusCode::FORBIDDEN);
        assert_eq!(
            context
                .share_store
                .list_calendar_shares("user", "cal")
                .await
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            context
                .share_store
                .get_visibility("user", "cal")
                .await
                .unwrap(),
            Policy::default()
        );
        // A Referer of this host is enough
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/add"),
            Some(USER),
            &[
                ("Host", "localhost"),
                ("Referer", "http://localhost/frontend/user"),
            ],
            "sharee=other&access=read",
        )
        .await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    }

    #[rstest]
    #[tokio::test]
    async fn test_owner_manages_shares() {
        let (context, app) = setup().await;
        let page = ok(&app, "GET", SHARES, USER, "").await;
        assert!(page.body.contains("bob display (bob)"), "{}", page.body);
        assert!(page.body.contains("group.team"));
        assert!(page.body.contains("manages its members itself"));
        // user is no member of group.team: no member list
        assert!(!page.body.contains("Members now"), "{}", page.body);

        // Step 1 shows only what was typed: not whom a name or an address belongs to
        context
            .share_store
            .set_principal_email("other", Some("other@example.org"))
            .await
            .unwrap();
        let preview = call(
            &app,
            "POST",
            &format!("{SHARES}/preview"),
            Some(USER),
            SAME_SITE,
            "sharee=+Other%40example.org+&access=write",
        )
        .await;
        assert_eq!(preview.status, StatusCode::OK);
        assert!(
            preview
                .body
                .contains("With <strong>Other@example.org</strong>")
                && preview
                    .body
                    .contains(r#"name="sharee" value="Other@example.org""#)
                && preview.body.contains("read and write"),
            "{}",
            preview.body
        );
        for resolved in ["other display", "(other)", r#"value="other""#] {
            assert!(
                !preview.body.contains(resolved),
                "{resolved}: {}",
                preview.body
            );
        }
        assert!(
            context
                .share_store
                .list_calendar_shares("user", "cal")
                .await
                .unwrap()
                .len()
                == 2,
            "nothing shared yet"
        );
        let unknown = call(
            &app,
            "POST",
            &format!("{SHARES}/preview"),
            Some(USER),
            SAME_SITE,
            "sharee=nobody&access=read",
        )
        .await;
        assert!(
            unknown
                .body
                .contains("There is no user or group with this name or e-mail address."),
            "{}",
            unknown.body
        );
        // A name of one principal that is the address of another
        context
            .share_store
            .set_principal_email("erin", Some("bob"))
            .await
            .unwrap();
        let ambiguous = call(
            &app,
            "POST",
            &format!("{SHARES}/preview"),
            Some(USER),
            SAME_SITE,
            "sharee=bob&access=read",
        )
        .await;
        assert!(
            ambiguous
                .body
                .contains("This name and this e-mail address belong to different users."),
            "{}",
            ambiguous.body
        );
        context
            .share_store
            .set_principal_email("erin", None)
            .await
            .unwrap();

        // A group shows its members only to members
        add_principal(&context, "group.mine", PrincipalType::Group, None).await;
        context
            .principal_store
            .add_membership("user", "group.mine")
            .await
            .unwrap();
        context
            .principal_store
            .add_membership("bob", "group.mine")
            .await
            .unwrap();
        let preview = call(
            &app,
            "POST",
            &format!("{SHARES}/preview"),
            Some(USER),
            SAME_SITE,
            "sharee=group.mine&access=read",
        )
        .await;
        assert!(
            preview.body.contains("Members now") && preview.body.contains("bob"),
            "{}",
            preview.body
        );
        // Not for the group's address: the address must not tell whom it belongs to
        context
            .share_store
            .set_principal_email("group.mine", Some("mine@example.org"))
            .await
            .unwrap();
        let preview = call(
            &app,
            "POST",
            &format!("{SHARES}/preview"),
            Some(USER),
            SAME_SITE,
            "sharee=mine%40example.org&access=read",
        )
        .await;
        assert!(
            preview.body.contains("mine@example.org")
                && !preview.body.contains("Members now")
                && !preview.body.contains("group.mine"),
            "{}",
            preview.body
        );
        let preview = call(
            &app,
            "POST",
            &format!("{SHARES}/preview"),
            Some(USER),
            SAME_SITE,
            "sharee=group.team&access=read",
        )
        .await;
        assert!(
            preview.body.contains("manages its members itself")
                && !preview.body.contains("Members now"),
            "{}",
            preview.body
        );

        // Step 2 shares with whom the typed text belongs to now
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/add"),
            Some(USER),
            SAME_SITE,
            "sharee=Other%40example.org&access=write",
        )
        .await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER);
        let share = context
            .share_store
            .get_calendar_share("other", "cal_shared_by_user")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            (share.access, share.created_by.as_deref()),
            (Access::Write, Some("user"))
        );

        // Access change
        call(
            &app,
            "POST",
            &format!("{SHARES}/access"),
            Some(USER),
            SAME_SITE,
            "sharee=other&access=read",
        )
        .await;
        let share = context
            .share_store
            .get_calendar_share("other", "cal_shared_by_user")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(share.access, Access::Read);

        // Ending several at once
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/remove"),
            Some(USER),
            SAME_SITE,
            "sharee=other&sharee=bob",
        )
        .await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER);
        let left: Vec<_> = context
            .share_store
            .list_calendar_shares("user", "cal")
            .await
            .unwrap()
            .into_iter()
            .map(|s| s.sharee)
            .collect();
        assert_eq!(left, ["group.team"]);

        // A page from before the end changes no access and makes no share again
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/access"),
            Some(USER),
            SAME_SITE,
            "sharee=other&access=write",
        )
        .await;
        assert_eq!(answer.status, StatusCode::OK);
        assert!(
            answer.body.contains("This share does not exist any more."),
            "{}",
            answer.body
        );
        assert!(
            context
                .share_store
                .get_calendar_share("other", "cal_shared_by_user")
                .await
                .unwrap()
                .is_none()
        );
        let answer = call(
            &app,
            "POST",
            "/frontend/user/user/addressbook/book/shares/access",
            Some(USER),
            SAME_SITE,
            "sharee=other&access=write",
        )
        .await;
        assert!(
            answer.body.contains("This share does not exist any more."),
            "{}",
            answer.body
        );
        assert_eq!(
            context
                .share_store
                .list_addressbook_shares("user", "book")
                .await
                .unwrap()
                .len(),
            1
        );

        // Refusals are shown on the page
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/add"),
            Some(USER),
            SAME_SITE,
            "sharee=user&access=read",
        )
        .await;
        assert!(answer.body.contains("with its owner"), "{}", answer.body);

        // Addressbooks
        let answer = call(
            &app,
            "POST",
            "/frontend/user/user/addressbook/book/shares/add",
            Some(USER),
            SAME_SITE,
            "sharee=other&access=read",
        )
        .await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER);
        assert_eq!(
            context
                .share_store
                .list_addressbook_shares("user", "book")
                .await
                .unwrap()
                .len(),
            2
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_sharees_cannot_manage() {
        let (context, app) = setup().await;
        assert_eq!(
            call(&app, "GET", SHARES, Some(BOB), &[], "").await.status,
            StatusCode::FORBIDDEN
        );
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/add"),
            Some(BOB),
            SAME_SITE,
            "sharee=other&access=write",
        )
        .await;
        assert_eq!(answer.status, StatusCode::FORBIDDEN);
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/remove"),
            Some(ERIN),
            SAME_SITE,
            "sharee=group.team",
        )
        .await;
        assert_eq!(answer.status, StatusCode::FORBIDDEN);
        assert_eq!(
            context
                .share_store
                .list_calendar_shares("user", "cal")
                .await
                .unwrap()
                .len(),
            2
        );
        // No page of a share's owner for the sharee
        let page = ok(&app, "GET", "/frontend/user/bob/sharing", BOB, "").await;
        assert!(!page.body.contains("/shares"), "{}", page.body);
    }

    #[rstest]
    #[tokio::test]
    async fn test_shared_with_me() {
        let (context, app) = setup().await;
        let page = ok(&app, "GET", "/frontend/user/bob/sharing", BOB, "").await;
        assert!(page.body.contains("Shared by user"), "{}", page.body);
        assert!(!page.body.contains("objects"), "no object counts or sizes");

        // The owner's color is unchecked: only a hex color reaches the page
        for (color, shown) in [
            ("red;background:url(https://evil.example/x)", "transparent"),
            ("#FFF8DCFF", "#FFF8DCFF"),
        ] {
            ok(
                &app,
                "PROPPATCH",
                OWN_CAL,
                USER,
                &format!(
                    r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propertyupdate xmlns:D="DAV:" xmlns:I="http://apple.com/ns/ical/">
  <D:set><D:prop><I:calendar-color>{color}</I:calendar-color></D:prop></D:set>
</D:propertyupdate>"#
                ),
            )
            .await;
            let page = ok(&app, "GET", "/frontend/user/bob/sharing", BOB, "").await;
            assert!(!page.body.contains("evil"), "{}", page.body);
            assert!(
                page.body.contains(&format!(r#"style="--color: {shown}""#)),
                "{}",
                page.body
            );
        }

        let answer = call(
            &app,
            "POST",
            "/frontend/user/bob/sharing/view",
            Some(BOB),
            SAME_SITE,
            "kind=calendar&home=bob&name=cal_shared_by_user&displayname=Mine&own_color=on&color=%23ff0000",
        )
        .await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
        let share = context
            .share_store
            .get_calendar_share("bob", "cal_shared_by_user")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            (share.displayname.as_deref(), share.color.as_deref()),
            (Some("Mine"), Some("#ff0000"))
        );
        // Group shares in erin's group home; bob cannot act in it
        let answer = call(
            &app,
            "POST",
            "/frontend/user/bob/sharing/remove",
            Some(BOB),
            SAME_SITE,
            "kind=calendar&home=group.team&name=cal_shared_by_user",
        )
        .await;
        assert_eq!(answer.status, StatusCode::FORBIDDEN);
        let page = ok(&app, "GET", "/frontend/user/erin/sharing", ERIN, "").await;
        assert!(page.body.contains("group.team/"), "{}", page.body);
        let answer = call(
            &app,
            "POST",
            "/frontend/user/erin/sharing/remove",
            Some(ERIN),
            SAME_SITE,
            "kind=calendar&home=group.team&name=cal_shared_by_user",
        )
        .await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER);
        assert!(
            context
                .share_store
                .get_calendar_share("group.team", "cal_shared_by_user")
                .await
                .unwrap()
                .is_none()
        );
        let answer = call(
            &app,
            "POST",
            "/frontend/user/bob/sharing/remove",
            Some(BOB),
            SAME_SITE,
            "kind=addressbook&home=bob&name=book_shared_by_user",
        )
        .await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER);
        assert!(
            context
                .share_store
                .list_addressbook_shares("user", "book")
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_visibility_levels() {
        let (context, app) = setup().await;
        // Per calendar, on its shares page; the user's sharing page has none
        let page = ok(&app, "GET", SHARES, USER, "").await;
        assert!(page.body.contains("Busy (time only)"), "{}", page.body);
        let section = ok(&app, "GET", "/frontend/user/user/sharing", USER, "").await;
        assert!(
            !section.body.contains("Busy (time only)"),
            "{}",
            section.body
        );
        let visibility = |context: TestStoreContext| async move {
            context
                .share_store
                .get_visibility("user", "cal")
                .await
                .unwrap()
        };
        // CONFIDENTIAL never above PRIVATE
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/visibility"),
            Some(USER),
            SAME_SITE,
            "private=1&confidential=3",
        )
        .await;
        assert!(
            answer
                .body
                .contains("Confidential objects may not show more than private ones."),
            "{}",
            answer.body
        );
        assert_eq!(visibility(context.clone()).await, Policy::default());
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/visibility"),
            Some(USER),
            SAME_SITE,
            "private=3&confidential=2",
        )
        .await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER);
        assert_eq!(
            visibility(context.clone()).await,
            Policy::new(3, 2).unwrap()
        );
        // Only the owner sets it, and only for a calendar
        let answer = call(
            &app,
            "POST",
            &format!("{SHARES}/visibility"),
            Some(BOB),
            SAME_SITE,
            "private=4&confidential=4",
        )
        .await;
        assert!(answer.status.is_client_error(), "{}", answer.status);
        assert_eq!(
            visibility(context.clone()).await,
            Policy::new(3, 2).unwrap()
        );
        let answer = call(
            &app,
            "POST",
            "/frontend/user/user/addressbook/book/shares/visibility",
            Some(USER),
            SAME_SITE,
            "private=4&confidential=4",
        )
        .await;
        assert_eq!(answer.status, StatusCode::NOT_FOUND);
    }
}

/// What a sharee's client asks to show who shares (DAVx5 and Apple look up `DAV:owner`)
const PRINCIPAL_PROPS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<propfind xmlns="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav" xmlns:CR="urn:ietf:params:xml:ns:carddav">
  <prop><displayname/><resourcetype/><principal-URL/><C:calendar-home-set/><CR:addressbook-home-set/><group-membership/><group-member-set/></prop>
</propfind>"#;

#[rstest]
#[tokio::test]
async fn test_sharees_see_who_shares() {
    let (context, app) = setup().await;
    context
        .principal_store
        .add_membership("user", "group.team")
        .await
        .unwrap();
    let propfind = |uri: &'static str, auth: (&'static str, &'static str), depth: &'static str| {
        let app = app.clone();
        async move {
            call(
                &app,
                "PROPFIND",
                uri,
                Some(auth),
                &[("Depth", depth)],
                PRINCIPAL_PROPS,
            )
            .await
        }
    };

    // The owner sees its homes and groups
    let own = propfind("/caldav/principal/user", USER, "0").await;
    assert_eq!(own.status, StatusCode::MULTI_STATUS);
    assert!(
        own.body.contains("/caldav/principal/group.team"),
        "{}",
        own.body
    );

    // A sharee (directly, through a group, over the Apple path) sees the name only: no homes,
    // no groups, no collections
    for (uri, auth) in [
        ("/caldav/principal/user", BOB),
        ("/caldav-compat/principal/user", BOB),
        ("/caldav/principal/user", ERIN),
        ("/carddav/principal/user", BOB),
    ] {
        for depth in ["0", "1"] {
            let answer = propfind(uri, auth, depth).await;
            assert_eq!(
                answer.status,
                StatusCode::MULTI_STATUS,
                "{uri} as {}",
                auth.0
            );
            assert!(answer.body.contains("<displayname>"), "{}", answer.body);
            assert!(
                !answer.body.contains("group.team"),
                "{uri} as {}: {}",
                auth.0,
                answer.body
            );
            assert_eq!(
                answer.body.matches("<href>").count(),
                2,
                "{uri} as {} depth {depth}: the response and the principal URL only: {}",
                auth.0,
                answer.body
            );
            assert_eq!(
                answer.body.matches("<response>").count(),
                1,
                "{}",
                answer.body
            );
        }
    }
    // Not shared with: nothing (erin's group has the calendar, not the addressbook)
    for (uri, auth) in [
        ("/carddav/principal/user", ERIN),
        ("/caldav/principal/user", OTHER),
        ("/carddav/principal/user", OTHER),
    ] {
        assert_eq!(
            propfind(uri, auth, "0").await.status,
            StatusCode::FORBIDDEN,
            "{uri} as {}",
            auth.0
        );
    }
    // And no more after the share ended
    context
        .share_store
        .unshare_calendar("user", "cal", "bob")
        .await
        .unwrap();
    assert_eq!(
        propfind("/caldav/principal/user", BOB, "0").await.status,
        StatusCode::FORBIDDEN
    );
    // Writing stays the owner's
    let answer = call(
        &app,
        "PROPPATCH",
        "/caldav/principal/user",
        Some(ERIN),
        &[],
        PROPPATCH_DISPLAYNAME,
    )
    .await;
    assert_eq!(answer.status, StatusCode::FORBIDDEN);
}
