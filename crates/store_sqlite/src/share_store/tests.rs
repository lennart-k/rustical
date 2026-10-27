use crate::tests::{TestStoreContext, test_store_context};
use rstest::rstest;
use rustical_ical::{AddressObject, CalendarObject, CalendarObjectType};
use rustical_store::auth::{AuthenticationProvider, Principal, PrincipalType};
use rustical_store::{
    Access, Addressbook, AddressbookReadStore, AddressbookWriteStore, Calendar, CalendarMetadata,
    CalendarReadStore, CalendarWriteStore, Error, ShareRejection, ShareStore, ShareViewUpdate,
};

async fn principal(context: &TestStoreContext, id: &str, principal_type: PrincipalType) {
    context
        .principal_store
        .insert_principal(
            Principal {
                id: id.to_owned(),
                displayname: None,
                principal_type,
                password: None,
                memberships: vec![],
            },
            false,
        )
        .await
        .unwrap();
}

fn calendar(principal: &str, id: &str) -> Calendar {
    Calendar {
        principal: principal.to_owned(),
        id: id.to_owned(),
        meta: CalendarMetadata {
            displayname: Some(format!("{id} of {principal}")),
            order: 0,
            description: None,
            color: Some("#123456".to_owned()),
        },
        timezone_id: None,
        deleted_at: None,
        synctoken: 0,
        subscription_url: None,
        push_topic: uuid::Uuid::new_v4().to_string(),
        components: vec![CalendarObjectType::Event],
    }
}

fn addressbook(principal: &str, id: &str) -> Addressbook {
    Addressbook {
        id: id.to_owned(),
        principal: principal.to_owned(),
        displayname: Some(format!("{id} of {principal}")),
        description: None,
        deleted_at: None,
        synctoken: 0,
        push_topic: uuid::Uuid::new_v4().to_string(),
    }
}

/// `user` (from the context), `bob`, `erin`, `group.team` = {erin}; `user` owns `cal` and `book`
async fn setup() -> TestStoreContext {
    let context = test_store_context().await;
    principal(&context, "bob", PrincipalType::Individual).await;
    principal(&context, "erin", PrincipalType::Individual).await;
    principal(&context, "group.team", PrincipalType::Group).await;
    context
        .principal_store
        .add_membership("erin", "group.team")
        .await
        .unwrap();
    context
        .cal_store
        .insert_calendar(calendar("user", "cal"))
        .await
        .unwrap();
    context
        .addr_store
        .insert_addressbook(addressbook("user", "book"))
        .await
        .unwrap();
    context
}

fn rejected(result: Result<impl std::fmt::Debug, Error>, rejection: ShareRejection) {
    match result {
        Err(Error::ShareRejected(got)) => assert_eq!(got, rejection),
        other => panic!("expected {rejection:?}, got {other:?}"),
    }
}

#[rstest]
#[tokio::test]
async fn test_share_upsert_keeps_view() {
    let context = setup().await;
    let shares = &context.share_store;

    let share = shares
        .share_calendar("user", "cal", "bob", Access::Read, Some("user"))
        .await
        .unwrap();
    assert_eq!(share.name, "cal_shared_by_user");
    assert_eq!(share.access, Access::Read);
    assert_eq!(share.created_by.as_deref(), Some("user"));

    shares
        .update_calendar_share_view(
            "bob",
            "cal_shared_by_user",
            ShareViewUpdate {
                displayname: Some(Some("Bobs name".to_owned())),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    shares
        .update_calendar_share_view(
            "bob",
            "cal_shared_by_user",
            ShareViewUpdate {
                color: Some(Some("#ff0000".to_owned())),
                order: Some(Some(3)),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    // Sharing again changes only the access
    let share = shares
        .share_calendar("user", "cal", "bob", Access::Write, None)
        .await
        .unwrap();
    assert_eq!(share.name, "cal_shared_by_user");
    assert_eq!(share.access, Access::Write);
    assert_eq!(share.displayname.as_deref(), Some("Bobs name"));
    assert_eq!(share.color.as_deref(), Some("#ff0000"));
    assert_eq!(share.order, Some(3));

    // A view update writes only what it names; Some(None) falls back to the owner's value
    shares
        .update_calendar_share_view(
            "bob",
            "cal_shared_by_user",
            ShareViewUpdate {
                displayname: Some(None),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let share = shares
        .get_calendar_share("bob", "cal_shared_by_user")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(share.displayname, None);
    assert_eq!(share.color.as_deref(), Some("#ff0000"));
    assert!(matches!(
        shares
            .update_calendar_share_view("bob", "missing", ShareViewUpdate::default())
            .await,
        Err(Error::NotFound)
    ));

    assert_eq!(
        shares
            .list_calendar_shares("user", "cal")
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(shares.calendars_shared_with("bob").await.unwrap().len(), 1);

    shares.unshare_calendar("user", "cal", "bob").await.unwrap();
    assert!(
        shares
            .list_calendar_shares("user", "cal")
            .await
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        shares.unshare_calendar("user", "cal", "bob").await,
        Err(Error::NotFound)
    ));
}

#[rstest]
#[tokio::test]
async fn test_share_rejections() {
    let context = setup().await;
    let shares = &context.share_store;

    rejected(
        shares
            .share_calendar("user", "cal", "user", Access::Read, None)
            .await,
        ShareRejection::Owner,
    );
    rejected(
        shares
            .share_calendar("user", "cal", "nobody", Access::Read, None)
            .await,
        ShareRejection::UnknownSharee,
    );
    // erin is a member of group.team, which already has full access to its own calendars
    context
        .cal_store
        .insert_calendar(calendar("group.team", "teamcal"))
        .await
        .unwrap();
    rejected(
        shares
            .share_calendar("group.team", "teamcal", "erin", Access::Read, None)
            .await,
        ShareRejection::OwnerMember,
    );
    // A group that contains the owner is fine
    context
        .principal_store
        .add_membership("user", "group.team")
        .await
        .unwrap();
    shares
        .share_calendar("user", "cal", "group.team", Access::Read, None)
        .await
        .unwrap();
    // Sharing with a group the owner is not a member of is fine too
    context
        .principal_store
        .remove_membership("user", "group.team")
        .await
        .unwrap();
    shares
        .share_addressbook("user", "book", "group.team", Access::Read, None)
        .await
        .unwrap();

    // The same text names one principal and is the e-mail address of another
    shares
        .set_principal_email("erin", Some("bob"))
        .await
        .unwrap();
    rejected(
        shares
            .share_calendar("user", "cal", "bob", Access::Read, None)
            .await,
        ShareRejection::AmbiguousSharee,
    );

    // Birthday calendars and subscriptions
    rejected(
        shares
            .share_calendar("user", "_birthdays_book", "erin", Access::Read, None)
            .await,
        ShareRejection::NotShareable,
    );
    let mut subscription = calendar("user", "sub");
    subscription.subscription_url = Some("https://example.com/feed.ics".to_owned());
    context
        .cal_store
        .insert_calendar(subscription)
        .await
        .unwrap();
    rejected(
        shares
            .share_calendar("user", "sub", "erin", Access::Read, None)
            .await,
        ShareRejection::NotShareable,
    );
    // Unknown or trashed collection
    assert!(matches!(
        shares
            .share_calendar("user", "missing", "erin", Access::Read, None)
            .await,
        Err(Error::NotFound)
    ));
    context
        .cal_store
        .delete_calendar("user", "cal", true)
        .await
        .unwrap();
    assert!(matches!(
        shares
            .share_calendar("user", "cal", "erin", Access::Read, None)
            .await,
        Err(Error::NotFound)
    ));
    assert!(matches!(
        shares
            .share_addressbook("user", "missing", "erin", Access::Read, None)
            .await,
        Err(Error::NotFound)
    ));
}

#[rstest]
#[tokio::test]
async fn test_share_by_email() {
    let context = setup().await;
    let shares = &context.share_store;

    shares
        .set_principal_email("bob", Some("Bob@Example.org"))
        .await
        .unwrap();
    assert_eq!(
        shares.principal_email("bob").await.unwrap().as_deref(),
        Some("bob@example.org")
    );
    assert_eq!(
        shares
            .resolve_email("BOB@example.ORG")
            .await
            .unwrap()
            .as_deref(),
        Some("bob")
    );
    let share = shares
        .share_calendar("user", "cal", " bob@EXAMPLE.org ", Access::Read, None)
        .await
        .unwrap();
    assert_eq!(share.sharee, "bob");

    // The newest claim wins
    shares
        .set_principal_email("erin", Some("bob@example.org"))
        .await
        .unwrap();
    assert_eq!(
        shares
            .resolve_email("bob@example.org")
            .await
            .unwrap()
            .as_deref(),
        Some("erin")
    );
    assert_eq!(shares.principal_email("bob").await.unwrap(), None);

    shares.set_principal_email("erin", None).await.unwrap();
    assert_eq!(shares.resolve_email("bob@example.org").await.unwrap(), None);
}

#[rstest]
#[tokio::test]
async fn test_share_names_never_collide() {
    let context = setup().await;
    let shares = &context.share_store;

    // bob already has a calendar (in the trash bin) with the generated name
    context
        .cal_store
        .insert_calendar(calendar("bob", "cal_shared_by_user"))
        .await
        .unwrap();
    context
        .cal_store
        .delete_calendar("bob", "cal_shared_by_user", true)
        .await
        .unwrap();
    let share = shares
        .share_calendar("user", "cal", "bob", Access::Read, None)
        .await
        .unwrap();
    assert_eq!(share.name, "cal_shared_by_user-2");

    // Creating a collection under a share's name is refused: MKCALENDAR, IMPORT, MKCOL
    assert!(matches!(
        context
            .cal_store
            .insert_calendar(calendar("bob", "cal_shared_by_user-2"))
            .await,
        Err(Error::AlreadyExists)
    ));
    assert!(matches!(
        context
            .cal_store
            .import_calendar(calendar("bob", "cal_shared_by_user-2"), vec![], true)
            .await,
        Err(Error::AlreadyExists)
    ));
    let share = shares
        .share_addressbook("user", "book", "bob", Access::Read, None)
        .await
        .unwrap();
    assert_eq!(share.name, "book_shared_by_user");
    assert!(matches!(
        context
            .addr_store
            .insert_addressbook(addressbook("bob", "book_shared_by_user"))
            .await,
        Err(Error::AlreadyExists)
    ));
    assert!(matches!(
        context
            .addr_store
            .import_addressbook(addressbook("bob", "book_shared_by_user"), vec![], true)
            .await,
        Err(Error::AlreadyExists)
    ));
}

#[rstest]
#[tokio::test]
async fn test_shares_follow_trash_and_deletion() {
    let context = setup().await;
    let shares = &context.share_store;
    shares
        .share_calendar("user", "cal", "bob", Access::Read, None)
        .await
        .unwrap();
    shares
        .share_addressbook("user", "book", "bob", Access::Write, None)
        .await
        .unwrap();

    // Trash: the share rests, it is neither listed nor found
    context
        .cal_store
        .delete_calendar("user", "cal", true)
        .await
        .unwrap();
    context
        .addr_store
        .delete_addressbook("user", "book", true)
        .await
        .unwrap();
    assert!(
        shares
            .calendars_shared_with("bob")
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        shares
            .addressbooks_shared_with("bob")
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        shares
            .get_calendar_share("bob", "cal_shared_by_user")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        shares
            .get_addressbook_share("bob", "book_shared_by_user")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        shares
            .list_calendar_shares("user", "cal")
            .await
            .unwrap()
            .len(),
        1
    );

    // Restore brings it back
    context
        .cal_store
        .restore_calendar("user", "cal")
        .await
        .unwrap();
    context
        .addr_store
        .restore_addressbook("user", "book")
        .await
        .unwrap();
    assert_eq!(shares.calendars_shared_with("bob").await.unwrap().len(), 1);
    assert_eq!(
        shares.addressbooks_shared_with("bob").await.unwrap().len(),
        1
    );

    // Purging the collection removes the share
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

    // Removing the sharee removes its shares
    context
        .principal_store
        .remove_principal("bob")
        .await
        .unwrap();
    assert!(
        shares
            .list_addressbook_shares("user", "book")
            .await
            .unwrap()
            .is_empty()
    );
}

/// Every write of the owner leaves the share rows alone. This runs with every fork sync, so it
/// also catches an upstream change that rebuilds or replaces the parent rows.
#[rstest]
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_owner_writes_keep_shares() {
    let context = setup().await;
    let shares = &context.share_store;
    shares
        .share_calendar("user", "cal", "bob", Access::Write, None)
        .await
        .unwrap();
    shares
        .share_addressbook("user", "book", "group.team", Access::Read, None)
        .await
        .unwrap();
    shares
        .update_calendar_share_view(
            "bob",
            "cal_shared_by_user",
            ShareViewUpdate {
                displayname: Some(Some("Mine".to_owned())),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let before = (
        shares.list_calendar_shares("user", "cal").await.unwrap(),
        shares
            .list_addressbook_shares("user", "book")
            .await
            .unwrap(),
    );

    // Calendar update (PROPPATCH), objects, import merge, trash and restore
    let mut changed = calendar("user", "cal");
    changed.meta.displayname = Some("Renamed".to_owned());
    changed.push_topic = context
        .cal_store
        .get_calendar("user", "cal", false)
        .await
        .unwrap()
        .push_topic;
    context
        .cal_store
        .update_calendar("user", "cal", changed.clone())
        .await
        .unwrap();
    context
        .cal_store
        .put_object("user", "cal", "obj", CalendarObject::example_1(), true)
        .await
        .unwrap();
    context
        .cal_store
        .import_calendar(
            changed,
            vec![CalendarObject::from_ics(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\nUID:imported\r\nDTSTAMP:20260101T000000Z\r\nDTSTART:20260105T100000Z\r\nSUMMARY:Imported\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
                    .to_owned(),
            )
            .unwrap()],
            true,
        )
        .await
        .unwrap();
    context
        .cal_store
        .delete_calendar("user", "cal", true)
        .await
        .unwrap();
    context
        .cal_store
        .restore_calendar("user", "cal")
        .await
        .unwrap();

    // Addressbook update, objects, import merge, trash and restore
    let mut changed = addressbook("user", "book");
    changed.displayname = Some("Renamed".to_owned());
    changed.push_topic = context
        .addr_store
        .get_addressbook("user", "book", false)
        .await
        .unwrap()
        .push_topic;
    context
        .addr_store
        .update_addressbook("user", "book", changed.clone())
        .await
        .unwrap();
    let card = AddressObject::from_vcf(
        "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:card\r\nFN:Card\r\nEND:VCARD\r\n".to_owned(),
    )
    .unwrap();
    AddressbookWriteStore::put_object(
        &context.addr_store,
        "user",
        "book",
        "card",
        card.clone(),
        true,
    )
    .await
    .unwrap();
    let card2 = AddressObject::from_vcf(
        "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:card2\r\nFN:Card 2\r\nEND:VCARD\r\n".to_owned(),
    )
    .unwrap();
    context
        .addr_store
        .import_addressbook(changed, vec![("card2".to_owned(), card2)], true)
        .await
        .unwrap();
    context
        .addr_store
        .delete_addressbook("user", "book", true)
        .await
        .unwrap();
    context
        .addr_store
        .restore_addressbook("user", "book")
        .await
        .unwrap();

    // Principal upserts and membership changes
    for id in ["user", "bob", "group.team"] {
        let mut existing = context
            .principal_store
            .get_principal(id)
            .await
            .unwrap()
            .unwrap();
        existing.displayname = Some("changed".to_owned());
        context
            .principal_store
            .insert_principal(existing, true)
            .await
            .unwrap();
    }
    for _ in 0..2 {
        context
            .principal_store
            .add_membership("bob", "group.team")
            .await
            .unwrap();
    }
    context
        .principal_store
        .remove_membership("bob", "group.team")
        .await
        .unwrap();
    let members = context
        .principal_store
        .clear_memberships("group.team")
        .await
        .unwrap();
    context
        .principal_store
        .set_memberships("group.team", &members)
        .await
        .unwrap();

    let after = (
        shares.list_calendar_shares("user", "cal").await.unwrap(),
        shares
            .list_addressbook_shares("user", "book")
            .await
            .unwrap(),
    );
    assert_eq!(before, after);
}

#[rstest]
#[tokio::test]
async fn test_membership_bulk_changes() {
    let context = setup().await;
    let principals = &context.principal_store;
    principals
        .add_membership("bob", "group.team")
        .await
        .unwrap();

    let mut members = principals.clear_memberships("group.team").await.unwrap();
    members.sort();
    assert_eq!(members, ["bob", "erin"]);
    assert!(
        principals
            .list_members("group.team")
            .await
            .unwrap()
            .is_empty()
    );

    // A principal deleted in the meantime is skipped
    principals.remove_principal("bob").await.unwrap();
    principals
        .set_memberships("group.team", &members)
        .await
        .unwrap();
    assert_eq!(
        principals.list_members("group.team").await.unwrap(),
        ["erin"]
    );
}

#[rstest]
#[tokio::test]
async fn test_visibility_table_rules() {
    let context = setup().await;
    let insert = |private: i64, confidential: i64| {
        let db = context.db.clone();
        async move {
            sqlx::query(
                "INSERT OR REPLACE INTO calendar_visibility (principal, cal_id, private, confidential) VALUES ('user', 'cal', ?, ?)",
            )
            .bind(private)
            .bind(confidential)
            .execute(&db)
            .await
        }
    };
    assert!(insert(1, 0).await.is_ok());
    assert!(insert(4, 4).await.is_ok());
    // CONFIDENTIAL never shows more than PRIVATE, levels are 0..=4
    assert!(insert(1, 2).await.is_err());
    assert!(insert(5, 0).await.is_err());
    assert!(insert(-1, -1).await.is_err());
    // Only for an existing calendar, and gone with it
    let orphan = sqlx::query(
        "INSERT INTO calendar_visibility (principal, cal_id, private, confidential) VALUES ('user', 'nope', 1, 0)",
    )
    .execute(&context.db)
    .await;
    assert!(orphan.is_err());
}
