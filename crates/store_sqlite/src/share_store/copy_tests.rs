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

fn copy(access: Access) -> Resolved {
    Resolved::for_tests(
        OWNER.to_owned(),
        CAL.to_owned(),
        ResolvedKind::Copy {
            access,
            policy: Policy::default(),
        },
    )
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
    // CONFIDENTIAL is hidden by default, PRIVATE shows only the time
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
    assert_eq!(
        shares.get_visibility(OWNER).await.unwrap(),
        Policy::default()
    );
    assert!(matches!(
        shares
            .set_visibility(
                OWNER,
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
    context
        .principal_store
        .insert_principal(
            Principal {
                id: "bob".to_owned(),
                displayname: None,
                principal_type: PrincipalType::Individual,
                password: None,
                memberships: vec![],
            },
            false,
        )
        .await
        .unwrap();
    shares
        .share_calendar(OWNER, CAL, "bob", Access::Read, None)
        .await
        .unwrap();
    let before = context
        .cal_store
        .get_calendar(OWNER, CAL, false)
        .await
        .unwrap()
        .synctoken;
    let policy = Policy::new(3, 2).unwrap();
    shares.set_visibility(OWNER, policy).await.unwrap();
    assert_eq!(shares.get_visibility(OWNER).await.unwrap(), policy);
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
    let resolved = Resolved::for_tests(
        OWNER.to_owned(),
        "book".to_owned(),
        ResolvedKind::Copy {
            access: Access::Write,
            policy: Policy::default(),
        },
    );
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
