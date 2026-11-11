//! Reading and writing through a share (`rustical_store::shared` on the SQLite stores).
use crate::tests::{TestStoreContext, test_store_context};
use chrono::NaiveDate;
use rstest::rstest;
use rustical_ical::visibility::Policy;
use rustical_ical::{AddressObject, CalendarObject, CalendarObjectType};
use rustical_store::access::{Resolved, ResolvedKind};
use rustical_store::auth::{AuthenticationProvider, Principal, PrincipalType};
use rustical_store::calendar_store::CalendarQuery;
use rustical_store::shared::{ETag, IfMatch, IfNoneMatch};
use rustical_store::{
    Access, Addressbook, AddressbookReadStore, AddressbookWriteStore, Calendar, CalendarMetadata,
    CalendarReadStore, CalendarWriteStore, CopyRule, Error, Preconditions, ShareStore, shared,
};

const OWNER: &str = "user";
const CAL: &str = "cal";
/// The sharee of `cal` and `book`, with write access
const SHAREE: &str = "bob";

fn ics(uid: &str, extra: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\nUID:{uid}\r\n\
         DTSTAMP:20260101T000000Z\r\nDTSTART:20260105T100000Z\r\nDTEND:20260105T110000Z\r\n\
         {extra}END:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

fn object(uid: &str, extra: &str) -> CalendarObject {
    CalendarObject::from_ics(ics(uid, extra)).unwrap()
}

const ALARM: &str =
    "BEGIN:VALARM\r\nACTION:DISPLAY\r\nDESCRIPTION:Owner alarm\r\nTRIGGER:-PT15M\r\nEND:VALARM\r\n";
const SHAREE_ALARM: &str =
    "BEGIN:VALARM\r\nACTION:DISPLAY\r\nDESCRIPTION:Sharee alarm\r\nTRIGGER:-PT5M\r\nEND:VALARM\r\n";

/// `cal` as the path of the sharee resolves it, with `access`
fn copy(access: Access) -> Resolved {
    Resolved::for_tests(
        OWNER.to_owned(),
        CAL.to_owned(),
        ResolvedKind::Copy {
            access,
            policy: Policy::new(1, 0).unwrap(),
        },
    )
    .in_home_for_tests(SHAREE.to_owned())
}

/// `book` as the path of the sharee resolves it, with `access`
fn book_copy(access: Access) -> Resolved {
    Resolved::for_tests(
        OWNER.to_owned(),
        "book".to_owned(),
        ResolvedKind::Copy {
            access,
            policy: Policy::default(),
        },
    )
    .in_home_for_tests(SHAREE.to_owned())
}

async fn setup() -> TestStoreContext {
    let context = test_store_context().await;
    context
        .cal_store
        .insert_calendar(Calendar {
            principal: OWNER.to_owned(),
            id: CAL.to_owned(),
            meta: CalendarMetadata::default(),
            timezone_id: None,
            deleted_at: None,
            synctoken: 0,
            subscription_url: None,
            push_topic: "topic".to_owned(),
            components: vec![CalendarObjectType::Event, CalendarObjectType::Todo],
        })
        .await
        .unwrap();
    context
        .addr_store
        .insert_addressbook(Addressbook {
            id: "book".to_owned(),
            principal: OWNER.to_owned(),
            displayname: None,
            description: None,
            deleted_at: None,
            synctoken: 0,
            push_topic: "book-topic".to_owned(),
        })
        .await
        .unwrap();
    // The tests read PRIVATE as busy (the calendar's setting; without one both are hidden)
    context
        .share_store
        .set_visibility(OWNER, CAL, Policy::new(1, 0).unwrap())
        .await
        .unwrap();
    context
        .principal_store
        .insert_principal(
            Principal {
                id: SHAREE.to_owned(),
                displayname: None,
                principal_type: PrincipalType::Individual,
                password: None,
                memberships: vec![],
            },
            false,
        )
        .await
        .unwrap();
    context
        .share_store
        .share_calendar(OWNER, CAL, SHAREE, Access::Write, None)
        .await
        .unwrap();
    context
        .share_store
        .share_addressbook(OWNER, "book", SHAREE, Access::Write, None)
        .await
        .unwrap();
    context
}

async fn owner_put(context: &TestStoreContext, id: &str, object: CalendarObject) {
    context
        .cal_store
        .put_object(OWNER, CAL, id, object, true)
        .await
        .unwrap();
}

async fn stored(context: &TestStoreContext, id: &str) -> String {
    context
        .cal_store
        .get_object(OWNER, CAL, id, true)
        .await
        .unwrap()
        .get_ics()
        .to_owned()
}

fn if_match(etag: &str) -> IfMatch {
    IfMatch::from(etag.parse::<ETag>().unwrap())
}

fn copy_rule(result: Result<impl std::fmt::Debug, Error>, rule: CopyRule) {
    match result {
        Err(Error::CopyRule(got)) => assert_eq!(got, rule),
        other => panic!("expected {rule:?}, got {other:?}"),
    }
}

#[rstest]
#[tokio::test]
async fn test_reads_show_views_only() {
    let context = setup().await;
    owner_put(
        &context,
        "public",
        object("public", &format!("SUMMARY:Lunch\r\n{ALARM}")),
    )
    .await;
    owner_put(
        &context,
        "private",
        object("private", "CLASS:PRIVATE\r\nSUMMARY:Doctor\r\n"),
    )
    .await;
    owner_put(
        &context,
        "confidential",
        object("confidential", "CLASS:CONFIDENTIAL\r\nSUMMARY:Secret\r\n"),
    )
    .await;
    let resolved = copy(Access::Read);

    let objects = shared::objects(&context.cal_store, &resolved)
        .await
        .unwrap();
    let mut ids: Vec<_> = objects.iter().map(|(id, _)| id.as_str()).collect();
    ids.sort_unstable();
    // The setup's levels: CONFIDENTIAL hidden, PRIVATE only the time
    assert_eq!(ids, ["private", "public"]);
    for (_, object) in &objects {
        let ics = object.get_ics();
        assert!(!ics.contains("Doctor") && !ics.contains("Secret"), "{ics}");
        assert!(!ics.contains("VALARM"), "{ics}");
    }
    assert!(matches!(
        shared::object(&context.cal_store, &resolved, "confidential", false).await,
        Err(Error::NotFound)
    ));
    // A query returns views: filters run on what the sharee sees
    let found = shared::query(&context.cal_store, &resolved, CalendarQuery::default())
        .await
        .unwrap();
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|(_, o)| !o.get_ics().contains("Doctor")));

    // The owner's own view is unchanged
    let own = shared::objects(
        &context.cal_store,
        &Resolved::for_tests(OWNER.to_owned(), CAL.to_owned(), ResolvedKind::Own),
    )
    .await
    .unwrap();
    assert_eq!(own.len(), 3);
}

#[rstest]
#[tokio::test]
async fn test_sync_reports_hidden_as_removed() {
    let context = setup().await;
    owner_put(&context, "a", object("a", "SUMMARY:A\r\n")).await;
    let resolved = copy(Access::Read);
    let (_, _, token) = shared::sync_changes(&context.cal_store, &resolved, 0)
        .await
        .unwrap();
    owner_put(
        &context,
        "a",
        object("a", "CLASS:CONFIDENTIAL\r\nSUMMARY:A\r\n"),
    )
    .await;
    let (changed, removed, _) = shared::sync_changes(&context.cal_store, &resolved, token)
        .await
        .unwrap();
    assert!(changed.is_empty());
    assert_eq!(removed, ["a"]);
}

#[rstest]
#[tokio::test]
async fn test_write_keeps_owner_alarms() {
    let context = setup().await;
    let resolved = copy(Access::Write);
    owner_put(
        &context,
        "e",
        object("e", &format!("SUMMARY:Old\r\n{ALARM}")),
    )
    .await;

    // The sharee sees no alarm; its view's ETag is what If-Match carries
    let seen = shared::object(&context.cal_store, &resolved, "e", false)
        .await
        .unwrap();
    assert!(!seen.get_ics().contains("VALARM"));
    // The ETag of the stored object (with the alarm) is not the sharee's: 412
    let raw_etag = context
        .cal_store
        .get_object(OWNER, CAL, "e", false)
        .await
        .unwrap()
        .get_etag();
    let raw_if_match = if_match(&raw_etag);
    assert!(matches!(
        shared::put_object(
            &context.cal_store,
            &context.share_store,
            &resolved,
            "e",
            object("e", "SUMMARY:New\r\n"),
            true,
            Preconditions {
                if_match: Some(&raw_if_match),
                if_none_match: None,
            },
        )
        .await,
        Err(Error::PreconditionFailed)
    ));

    let view_if_match = if_match(&seen.get_etag());
    let etag = shared::put_object(
        &context.cal_store,
        &context.share_store,
        &resolved,
        "e",
        object("e", &format!("SUMMARY:New\r\n{SHAREE_ALARM}")),
        true,
        Preconditions {
            if_match: Some(&view_if_match),
            if_none_match: None,
        },
    )
    .await
    .unwrap();
    // What the sharee sent (with its alarm) is not what is stored: no ETag
    assert_eq!(etag, None);
    let ics = stored(&context, "e").await;
    assert!(ics.contains("SUMMARY:New"), "{ics}");
    assert!(ics.contains("Owner alarm"), "{ics}");
    assert!(!ics.contains("Sharee alarm"), "{ics}");

    // Without an alarm in what is sent, the answer has the ETag of the new view
    let etag = shared::put_object(
        &context.cal_store,
        &context.share_store,
        &resolved,
        "e",
        object("e", "SUMMARY:Newer\r\n"),
        true,
        Preconditions::default(),
    )
    .await
    .unwrap()
    .unwrap();
    let seen = shared::object(&context.cal_store, &resolved, "e", false)
        .await
        .unwrap();
    assert_eq!(etag, seen.get_etag());
    assert!(stored(&context, "e").await.contains("Owner alarm"));
}

#[rstest]
#[tokio::test]
async fn test_new_exception_inherits_master_alarms() {
    let context = setup().await;
    let resolved = copy(Access::Write);
    owner_put(
        &context,
        "s",
        object(
            "s",
            &format!("RRULE:FREQ=DAILY;COUNT=3\r\nSUMMARY:Series\r\n{ALARM}"),
        ),
    )
    .await;
    let sent = CalendarObject::from_ics(format!(
        "{}BEGIN:VEVENT\r\nUID:s\r\nDTSTAMP:20260101T000000Z\r\nRECURRENCE-ID:20260106T100000Z\r\n\
         DTSTART:20260106T120000Z\r\nDTEND:20260106T130000Z\r\nSUMMARY:Moved\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        ics("s", "RRULE:FREQ=DAILY;COUNT=3\r\nSUMMARY:Series\r\n")
            .trim_end_matches("END:VCALENDAR\r\n")
    ))
    .unwrap();
    shared::put_object(
        &context.cal_store,
        &context.share_store,
        &resolved,
        "s",
        sent,
        true,
        Preconditions::default(),
    )
    .await
    .unwrap();
    let ics = stored(&context, "s").await;
    assert_eq!(ics.matches("Owner alarm").count(), 2, "{ics}");
}

#[rstest]
#[tokio::test]
async fn test_new_objects_get_no_alarms() {
    let context = setup().await;
    let resolved = copy(Access::Write);
    shared::put_object(
        &context.cal_store,
        &context.share_store,
        &resolved,
        "n",
        object("n", &format!("SUMMARY:New\r\n{SHAREE_ALARM}")),
        true,
        Preconditions::default(),
    )
    .await
    .unwrap();
    assert!(!stored(&context, "n").await.contains("VALARM"));
}

#[rstest]
#[tokio::test]
async fn test_write_rules() {
    let context = setup().await;
    let resolved = copy(Access::Write);
    let put = |id: &'static str, object: CalendarObject| {
        let context = context.clone();
        let resolved = resolved.clone();
        async move {
            shared::put_object(
                &context.cal_store,
                &context.share_store,
                &resolved,
                id,
                object,
                true,
                Preconditions::default(),
            )
            .await
        }
    };
    owner_put(&context, "live", object("live-uid", "SUMMARY:Live\r\n")).await;
    owner_put(
        &context,
        "trashed",
        object("trashed-uid", "SUMMARY:Trash\r\n"),
    )
    .await;
    context
        .cal_store
        .delete_object(OWNER, CAL, "trashed", true)
        .await
        .unwrap();
    owner_put(
        &context,
        "private",
        object("private-uid", "CLASS:PRIVATE\r\n"),
    )
    .await;

    // A new name with the UID of another object, live or in the trash bin
    copy_rule(
        put("new1", object("live-uid", "")).await,
        CopyRule::ForeignUid,
    );
    copy_rule(
        put("new2", object("trashed-uid", "")).await,
        CopyRule::ForeignUid,
    );
    // An existing name with another UID
    copy_rule(
        put("live", object("other-uid", "")).await,
        CopyRule::UidChanged,
    );
    // The name of an object in the owner's trash bin: never overwritten nor revived
    copy_rule(
        put("trashed", object("trashed-uid", "")).await,
        CopyRule::TrashHref,
    );
    assert!(
        context
            .cal_store
            .get_object(OWNER, CAL, "trashed", false)
            .await
            .is_err()
    );
    // Objects the sharee does not see in full
    copy_rule(
        put("private", object("private-uid", "")).await,
        CopyRule::Level,
    );
    // A read-only share writes nothing
    assert!(matches!(
        shared::put_object(
            &context.cal_store,
            &context.share_store,
            &copy(Access::Read),
            "new3",
            object("new3", ""),
            true,
            Preconditions::default(),
        )
        .await,
        Err(Error::Forbidden)
    ));
    // If-None-Match: * on an existing object
    let any = IfNoneMatch::any();
    assert!(matches!(
        shared::put_object(
            &context.cal_store,
            &context.share_store,
            &resolved,
            "live",
            object("live-uid", "SUMMARY:x\r\n"),
            true,
            Preconditions {
                if_match: None,
                if_none_match: Some(&any),
            },
        )
        .await,
        Err(Error::PreconditionFailed)
    ));

    // A sharee may make an object invisible to itself (and to all sharees)
    put(
        "live",
        object("live-uid", "CLASS:CONFIDENTIAL\r\nSUMMARY:Hidden\r\n"),
    )
    .await
    .unwrap();
    assert!(matches!(
        shared::object(&context.cal_store, &resolved, "live", false).await,
        Err(Error::NotFound)
    ));
    assert!(stored(&context, "live").await.contains("SUMMARY:Hidden"));
}

#[rstest]
#[tokio::test]
async fn test_races_with_the_owner() {
    let context = setup().await;
    let resolved = copy(Access::Write);
    owner_put(&context, "e", object("e", "SUMMARY:Shared\r\n")).await;
    let seen = shared::object(&context.cal_store, &resolved, "e", false)
        .await
        .unwrap();
    let seen_if_match = if_match(&seen.get_etag());
    let put = |with_etag: bool| {
        let context = context.clone();
        let resolved = resolved.clone();
        let seen_if_match = seen_if_match.clone();
        async move {
            shared::put_object(
                &context.cal_store,
                &context.share_store,
                &resolved,
                "e",
                object("e", "SUMMARY:Sharee\r\n"),
                true,
                Preconditions {
                    if_match: with_etag.then_some(&seen_if_match),
                    if_none_match: None,
                },
            )
            .await
        }
    };

    // The owner makes it confidential before the sharee writes: nothing is written back
    owner_put(
        &context,
        "e",
        object("e", "CLASS:CONFIDENTIAL\r\nSUMMARY:Shared\r\n"),
    )
    .await;
    copy_rule(put(true).await, CopyRule::Level);
    assert!(stored(&context, "e").await.contains("CLASS:CONFIDENTIAL"));

    // The owner changes it: the sharee's old ETag fails
    owner_put(&context, "e", object("e", "SUMMARY:Changed\r\n")).await;
    assert!(matches!(put(true).await, Err(Error::PreconditionFailed)));

    // The owner moves it to the trash bin: never revived
    context
        .cal_store
        .delete_object(OWNER, CAL, "e", true)
        .await
        .unwrap();
    copy_rule(put(false).await, CopyRule::TrashHref);
    assert!(
        context
            .cal_store
            .get_object(OWNER, CAL, "e", false)
            .await
            .is_err()
    );
}

#[rstest]
#[tokio::test]
async fn test_moved_object_is_found_in_its_new_time_range() {
    let context = setup().await;
    let resolved = copy(Access::Write);
    owner_put(&context, "m", object("m", "SUMMARY:Meeting\r\n")).await;
    let moved =
        CalendarObject::from_ics(ics("m", "SUMMARY:Meeting\r\n").replace("20260105", "20270105"))
            .unwrap();
    shared::put_object(
        &context.cal_store,
        &context.share_store,
        &resolved,
        "m",
        moved,
        true,
        Preconditions::default(),
    )
    .await
    .unwrap();
    let range = |year: i32| CalendarQuery {
        time_start: NaiveDate::from_ymd_opt(year, 1, 1),
        time_end: NaiveDate::from_ymd_opt(year, 1, 31),
    };
    assert_eq!(
        shared::query(&context.cal_store, &resolved, range(2027))
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        shared::query(&context.cal_store, &resolved, range(2026))
            .await
            .unwrap()
            .is_empty()
    );
}

#[rstest]
#[tokio::test]
async fn test_delete_through_a_share() {
    let context = setup().await;
    let resolved = copy(Access::Write);
    owner_put(&context, "e", object("e", "SUMMARY:x\r\n")).await;
    owner_put(&context, "p", object("p", "CLASS:PRIVATE\r\n")).await;
    owner_put(&context, "c", object("c", "CLASS:CONFIDENTIAL\r\n")).await;
    let delete = |id: &'static str| {
        let context = context.clone();
        let resolved = resolved.clone();
        async move {
            shared::delete_object(
                &context.cal_store,
                &context.share_store,
                &resolved,
                id,
                false,
                Preconditions::default(),
            )
            .await
        }
    };
    copy_rule(delete("p").await, CopyRule::Level);
    assert!(matches!(delete("c").await, Err(Error::NotFound)));
    // Always into the trash bin, even when asked for a hard delete
    delete("e").await.unwrap();
    assert!(
        context
            .cal_store
            .get_object(OWNER, CAL, "e", true)
            .await
            .is_ok()
    );
    assert!(
        context
            .cal_store
            .get_object(OWNER, CAL, "e", false)
            .await
            .is_err()
    );
}

#[rstest]
#[tokio::test]
async fn test_visibility_changes_are_synced() {
    let context = setup().await;
    let shares = &context.share_store;
    // No row (a calendar the setup did not set): hidden, both
    assert_eq!(
        shares.get_visibility(OWNER, "other").await.unwrap(),
        Policy::new(0, 0).unwrap()
    );
    assert_eq!(Policy::default(), Policy::new(0, 0).unwrap());
    assert!(matches!(
        shares
            .set_visibility(OWNER, "nope", Policy::new(1, 0).unwrap())
            .await,
        Err(Error::NotFound)
    ));
    assert!(matches!(
        shares
            .set_visibility(
                OWNER,
                CAL,
                Policy {
                    private: 1,
                    confidential: 2
                }
            )
            .await,
        Err(Error::InvalidVisibility)
    ));

    owner_put(&context, "public", object("public", "")).await;
    owner_put(&context, "private", object("private", "CLASS:PRIVATE\r\n")).await;
    owner_put(
        &context,
        "confidential",
        object("confidential", "CLASS:CONFIDENTIAL\r\n"),
    )
    .await;
    let before = context
        .cal_store
        .get_calendar(OWNER, CAL, false)
        .await
        .unwrap()
        .synctoken;
    let policy = Policy::new(3, 2).unwrap();
    shares.set_visibility(OWNER, CAL, policy).await.unwrap();
    assert_eq!(shares.get_visibility(OWNER, CAL).await.unwrap(), policy);
    // Per calendar: another one of the owner keeps its own (here: none set)
    assert_eq!(
        shares.get_visibility(OWNER, "other").await.unwrap(),
        Policy::default()
    );
    let after = context
        .cal_store
        .get_calendar(OWNER, CAL, false)
        .await
        .unwrap()
        .synctoken;
    // One change per object whose view depends on the policy
    assert_eq!(after - before, 2);
    let (changed, _, _) = context
        .cal_store
        .sync_changes(OWNER, CAL, before)
        .await
        .unwrap();
    let mut ids: Vec<_> = changed.iter().map(|(id, _)| id.as_str()).collect();
    ids.sort_unstable();
    assert_eq!(ids, ["confidential", "private"]);
}

#[rstest]
#[tokio::test]
async fn test_contacts_through_a_share() {
    let context = setup().await;
    let resolved = book_copy(Access::Write);
    let card = |name: &str| {
        AddressObject::from_vcf(format!(
            "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:c\r\nFN:{name}\r\nEND:VCARD\r\n"
        ))
        .unwrap()
    };
    let put = |id: &'static str, object: AddressObject| {
        let context = context.clone();
        let resolved = resolved.clone();
        async move {
            shared::put_address_object(
                &context.addr_store,
                &context.share_store,
                &resolved,
                id,
                object,
                true,
                Preconditions::default(),
            )
            .await
        }
    };
    put("c", card("First")).await.unwrap();
    put("c", card("Second")).await.unwrap();
    assert!(
        shared::address_object(&context.addr_store, &resolved, "c", false)
            .await
            .unwrap()
            .get_vcf()
            .contains("Second")
    );
    shared::delete_address_object(
        &context.addr_store,
        &context.share_store,
        &resolved,
        "c",
        false,
        Preconditions::default(),
    )
    .await
    .unwrap();
    // In the owner's trash bin, and never revived through the share
    assert!(
        AddressbookReadStore::get_object(&context.addr_store, OWNER, "book", "c", true)
            .await
            .is_ok()
    );
    copy_rule(put("c", card("Third")).await, CopyRule::TrashHref);
}

#[rstest]
#[tokio::test]
async fn test_read_shares_write_nothing() {
    let context = setup().await;
    owner_put(&context, "e", object("e", "SUMMARY:Kept\r\n")).await;
    let card = AddressObject::from_vcf(
        "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:c\r\nFN:Kept\r\nEND:VCARD\r\n".to_owned(),
    )
    .unwrap();
    AddressbookWriteStore::put_object(&context.addr_store, OWNER, "book", "c", card, true)
        .await
        .unwrap();
    let read = copy(Access::Read);
    let book = book_copy(Access::Read);
    let changed = |vcf: &str| {
        AddressObject::from_vcf(format!(
            "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:c\r\nFN:{vcf}\r\nEND:VCARD\r\n"
        ))
        .unwrap()
    };

    assert!(matches!(
        shared::put_objects(
            &context.cal_store,
            &context.share_store,
            &read,
            vec![
                ("e".to_owned(), object("e", "SUMMARY:Changed\r\n")),
                ("n".to_owned(), object("n", "")),
            ],
            true,
        )
        .await,
        Err(Error::Forbidden)
    ));
    assert!(matches!(
        shared::delete_object(
            &context.cal_store,
            &context.share_store,
            &read,
            "e",
            true,
            Preconditions::default(),
        )
        .await,
        Err(Error::Forbidden)
    ));
    assert!(matches!(
        shared::put_address_object(
            &context.addr_store,
            &context.share_store,
            &book,
            "c",
            changed("Changed"),
            true,
            Preconditions::default(),
        )
        .await,
        Err(Error::Forbidden)
    ));
    assert!(matches!(
        shared::delete_address_object(
            &context.addr_store,
            &context.share_store,
            &book,
            "c",
            true,
            Preconditions::default(),
        )
        .await,
        Err(Error::Forbidden)
    ));

    assert!(stored(&context, "e").await.contains("SUMMARY:Kept"));
    assert!(
        context
            .cal_store
            .get_object(OWNER, CAL, "n", true)
            .await
            .is_err()
    );
    assert!(
        AddressbookReadStore::get_object(&context.addr_store, OWNER, "book", "c", false)
            .await
            .unwrap()
            .get_vcf()
            .contains("FN:Kept")
    );
}

/// A path resolved before the owner changed the share writes nothing the share does not allow
/// now: the store checks the share again in the transaction of the write
#[rstest]
#[tokio::test]
async fn test_writes_check_the_share_they_go_through() {
    let context = setup().await;
    let shares = &context.share_store;
    owner_put(&context, "e", object("e", "SUMMARY:Kept\r\n")).await;
    let card = |name: &str| {
        AddressObject::from_vcf(format!(
            "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:c\r\nFN:{name}\r\nEND:VCARD\r\n"
        ))
        .unwrap()
    };
    AddressbookWriteStore::put_object(&context.addr_store, OWNER, "book", "c", card("Kept"), true)
        .await
        .unwrap();
    // Resolved while the share still wrote
    let resolved = copy(Access::Write);
    let book = book_copy(Access::Write);
    let attempts = |context: TestStoreContext, resolved: Resolved, book: Resolved| async move {
        [
            shared::put_object(
                &context.cal_store,
                &context.share_store,
                &resolved,
                "e",
                object("e", "SUMMARY:Changed\r\n"),
                true,
                Preconditions::default(),
            )
            .await
            .map(drop),
            shared::put_objects(
                &context.cal_store,
                &context.share_store,
                &resolved,
                vec![("n".to_owned(), object("n", ""))],
                true,
            )
            .await,
            shared::delete_object(
                &context.cal_store,
                &context.share_store,
                &resolved,
                "e",
                true,
                Preconditions::default(),
            )
            .await,
            shared::put_address_object(
                &context.addr_store,
                &context.share_store,
                &book,
                "c",
                card("Changed"),
                true,
                Preconditions::default(),
            )
            .await
            .map(drop),
            shared::delete_address_object(
                &context.addr_store,
                &context.share_store,
                &book,
                "c",
                true,
                Preconditions::default(),
            )
            .await,
        ]
    };

    // Now read only
    shares
        .set_calendar_share_access(OWNER, CAL, SHAREE, Access::Read)
        .await
        .unwrap();
    shares
        .set_addressbook_share_access(OWNER, "book", SHAREE, Access::Read)
        .await
        .unwrap();
    for result in attempts(context.clone(), resolved.clone(), book.clone()).await {
        assert!(matches!(result, Err(Error::Forbidden)), "{result:?}");
    }
    // Now gone
    shares.unshare_calendar(OWNER, CAL, SHAREE).await.unwrap();
    shares
        .unshare_addressbook(OWNER, "book", SHAREE)
        .await
        .unwrap();
    for result in attempts(context.clone(), resolved, book).await {
        assert!(matches!(result, Err(Error::NotFound)), "{result:?}");
    }
    // Another home than the share's never writes
    let in_owner_home = Resolved::for_tests(
        OWNER.to_owned(),
        CAL.to_owned(),
        ResolvedKind::Copy {
            access: Access::Write,
            policy: Policy::new(1, 0).unwrap(),
        },
    );
    let book_in_owner_home = Resolved::for_tests(
        OWNER.to_owned(),
        "book".to_owned(),
        ResolvedKind::Copy {
            access: Access::Write,
            policy: Policy::default(),
        },
    );
    for result in attempts(context.clone(), in_owner_home, book_in_owner_home).await {
        assert!(matches!(result, Err(Error::NotFound)), "{result:?}");
    }

    assert!(stored(&context, "e").await.contains("SUMMARY:Kept"));
    assert!(
        context
            .cal_store
            .get_object(OWNER, CAL, "n", true)
            .await
            .is_err()
    );
    assert!(
        AddressbookReadStore::get_object(&context.addr_store, OWNER, "book", "c", false)
            .await
            .unwrap()
            .get_vcf()
            .contains("FN:Kept")
    );
}

/// The conditions of a DELETE are checked in its transaction, on the sharee's view
#[rstest]
#[tokio::test]
async fn test_conditional_delete_through_a_share() {
    let context = setup().await;
    let resolved = copy(Access::Write);
    owner_put(
        &context,
        "e",
        object("e", &format!("SUMMARY:Old\r\n{ALARM}")),
    )
    .await;
    let old_view = shared::object(&context.cal_store, &resolved, "e", false)
        .await
        .unwrap()
        .get_etag();
    owner_put(
        &context,
        "e",
        object("e", &format!("SUMMARY:New\r\n{ALARM}")),
    )
    .await;
    let delete = |etag: String| {
        let context = context.clone();
        let resolved = resolved.clone();
        async move {
            let if_match = if_match(&etag);
            shared::delete_object(
                &context.cal_store,
                &context.share_store,
                &resolved,
                "e",
                true,
                Preconditions {
                    if_match: Some(&if_match),
                    if_none_match: None,
                },
            )
            .await
        }
    };
    assert!(matches!(
        delete(old_view).await,
        Err(Error::PreconditionFailed)
    ));
    // The stored ETag (with the owner's alarm) is not the sharee's either
    let raw = context
        .cal_store
        .get_object(OWNER, CAL, "e", false)
        .await
        .unwrap()
        .get_etag();
    assert!(matches!(delete(raw).await, Err(Error::PreconditionFailed)));
    assert!(stored(&context, "e").await.contains("SUMMARY:New"));
    let any = IfNoneMatch::any();
    assert!(matches!(
        shared::delete_object(
            &context.cal_store,
            &context.share_store,
            &resolved,
            "e",
            true,
            Preconditions {
                if_match: None,
                if_none_match: Some(&any),
            },
        )
        .await,
        Err(Error::PreconditionFailed)
    ));
    let view = shared::object(&context.cal_store, &resolved, "e", false)
        .await
        .unwrap()
        .get_etag();
    delete(view).await.unwrap();
    assert!(
        context
            .cal_store
            .get_object(OWNER, CAL, "e", false)
            .await
            .is_err()
    );

    // Contacts
    let book = book_copy(Access::Write);
    let card = |name: &str| {
        AddressObject::from_vcf(format!(
            "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:c\r\nFN:{name}\r\nEND:VCARD\r\n"
        ))
        .unwrap()
    };
    let old = card("Old");
    AddressbookWriteStore::put_object(&context.addr_store, OWNER, "book", "c", old.clone(), true)
        .await
        .unwrap();
    AddressbookWriteStore::put_object(&context.addr_store, OWNER, "book", "c", card("New"), true)
        .await
        .unwrap();
    let old_if_match = if_match(&old.get_etag());
    assert!(matches!(
        shared::delete_address_object(
            &context.addr_store,
            &context.share_store,
            &book,
            "c",
            true,
            Preconditions {
                if_match: Some(&old_if_match),
                if_none_match: None,
            },
        )
        .await,
        Err(Error::PreconditionFailed)
    ));
    assert!(
        AddressbookReadStore::get_object(&context.addr_store, OWNER, "book", "c", false)
            .await
            .is_ok()
    );
}

async fn synctoken(context: &TestStoreContext) -> i64 {
    context
        .cal_store
        .get_calendar(OWNER, CAL, false)
        .await
        .unwrap()
        .synctoken
}

/// Logged object ids since `token`, sorted
async fn logged_since(context: &TestStoreContext, token: i64) -> Vec<String> {
    let (changed, removed, _) = context
        .cal_store
        .sync_changes(OWNER, CAL, token)
        .await
        .unwrap();
    let mut ids: Vec<_> = changed
        .into_iter()
        .map(|(id, _)| id)
        .chain(removed)
        .collect();
    ids.sort_unstable();
    ids
}

#[rstest]
#[tokio::test]
async fn test_visibility_logs_only_changed_levels() {
    let context = setup().await;
    let shares = &context.share_store;
    owner_put(&context, "public", object("public", "CLASS:PUBLIC\r\n")).await;
    owner_put(&context, "plain", object("plain", "")).await;
    owner_put(&context, "private", object("private", "CLASS:PRIVATE\r\n")).await;
    owner_put(&context, "unknown", object("unknown", "CLASS:X-OTHER\r\n")).await;
    owner_put(
        &context,
        "confidential",
        object("confidential", "CLASS:CONFIDENTIAL\r\n"),
    )
    .await;
    owner_put(&context, "trashed", object("trashed", "CLASS:PRIVATE\r\n")).await;
    context
        .cal_store
        .delete_object(OWNER, CAL, "trashed", true)
        .await
        .unwrap();

    // The setup set (1, 0): the same again changes nothing
    let before = synctoken(&context).await;
    shares
        .set_visibility(OWNER, CAL, Policy::new(1, 0).unwrap())
        .await
        .unwrap();
    assert_eq!(synctoken(&context).await, before);

    // Only PRIVATE changes: private and unknown classes, never the trash bin
    shares
        .set_visibility(OWNER, CAL, Policy::new(2, 0).unwrap())
        .await
        .unwrap();
    assert_eq!(synctoken(&context).await, before + 2);
    assert_eq!(logged_since(&context, before).await, ["private", "unknown"]);

    // Only CONFIDENTIAL changes
    let before = synctoken(&context).await;
    shares
        .set_visibility(OWNER, CAL, Policy::new(2, 1).unwrap())
        .await
        .unwrap();
    assert_eq!(logged_since(&context, before).await, ["confidential"]);

    // A calendar without a setting: setting the default changes nothing
    context
        .cal_store
        .insert_calendar(Calendar {
            principal: OWNER.to_owned(),
            id: "fresh".to_owned(),
            meta: CalendarMetadata::default(),
            timezone_id: None,
            deleted_at: None,
            synctoken: 0,
            subscription_url: None,
            push_topic: "fresh-topic".to_owned(),
            components: vec![CalendarObjectType::Event],
        })
        .await
        .unwrap();
    context
        .cal_store
        .put_object(OWNER, "fresh", "p", object("p", "CLASS:PRIVATE\r\n"), true)
        .await
        .unwrap();
    let fresh = |context: TestStoreContext| async move {
        context
            .cal_store
            .get_calendar(OWNER, "fresh", false)
            .await
            .unwrap()
            .synctoken
    };
    let before = fresh(context.clone()).await;
    shares
        .set_visibility(OWNER, "fresh", Policy::default())
        .await
        .unwrap();
    assert_eq!(fresh(context.clone()).await, before);
}

/// A sharee whose share ends and is made again (with the same name) syncs with its old token
/// and learns what changed meanwhile, also through changes while nothing was shared
#[rstest]
#[tokio::test]
async fn test_visibility_change_while_unshared_reaches_the_old_token() {
    let context = setup().await;
    let shares = &context.share_store;
    shares
        .set_visibility(OWNER, CAL, Policy::new(4, 0).unwrap())
        .await
        .unwrap();
    owner_put(
        &context,
        "p",
        object("p", "CLASS:PRIVATE\r\nSUMMARY:Doctor\r\n"),
    )
    .await;
    let resolve = |context: TestStoreContext| async move {
        rustical_store::access::resolve_calendar(
            &context.share_store,
            &context.cal_store,
            SHAREE,
            "cal_shared_by_user",
            false,
        )
        .await
        .unwrap()
        .0
    };
    let resolved = resolve(context.clone()).await;
    let (changed, _, token) = shared::sync_changes(&context.cal_store, &resolved, 0)
        .await
        .unwrap();
    assert!(
        changed
            .iter()
            .any(|(id, object)| id == "p" && object.get_ics().contains("Doctor"))
    );

    shares.unshare_calendar(OWNER, CAL, SHAREE).await.unwrap();
    shares
        .set_visibility(OWNER, CAL, Policy::new(0, 0).unwrap())
        .await
        .unwrap();
    let share = shares
        .share_calendar(OWNER, CAL, SHAREE, Access::Write, None)
        .await
        .unwrap();
    assert_eq!(share.name, "cal_shared_by_user");

    let resolved = resolve(context.clone()).await;
    let (changed, removed, _) = shared::sync_changes(&context.cal_store, &resolved, token)
        .await
        .unwrap();
    assert!(changed.is_empty());
    assert_eq!(removed, ["p"]);
}
