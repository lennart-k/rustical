use std::collections::HashSet;

use rstest::rstest;
use rustical_ical::AddressObject;
use rustical_store::{
    Addressbook, AddressbookReadStore, AddressbookWriteStore,
    auth::{AuthenticationProvider, Principal, PrincipalType},
};

use crate::addressbook_store::PostgresAddressbookStore;
use crate::tests::{TestStoreContext, test_store_context};

#[rstest]
#[tokio::test]
async fn test_sync_full(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let addr_store = context.await.addr_store;

    let principal = "user".to_string();
    let addr_id = "addr".to_string();

    let addressbook = Addressbook {
        id: addr_id.clone(),
        principal: principal.clone(),
        displayname: None,
        description: None,
        deleted_at: None,
        synctoken: 0,
        push_topic: "alskdj".to_string(),
    };
    addr_store.insert_addressbook(addressbook).await.unwrap();

    let object = AddressObject::example_minimal();
    let id1 = "53fe5a9f-df12-478c-9ed2-1a01d83e4975";
    let id2 = "d3008843-1204-4b7e-98b8-f038328431fe";
    let id3 = "688a2ebf-fe3b-40f1-a67b-1a137552a831";
    let id4 = "61022a0d-b478-4116-98bc-a06da19dc568";

    // A list of operations where objects get removed that never existed and objects get recreated
    // https://github.com/lennart-k/rustical/issues/251
    let operations = [
        (true, id1, 1),
        (true, id2, 2),
        (false, id1, 3),
        (true, id1, 4),
        (true, id3, 5),
        (false, id4, 6),
    ];

    for (add, id, synctoken_after) in operations {
        if add {
            addr_store
                .put_object(&principal, &addr_id, id, object.clone(), true)
                .await
                .unwrap();
        } else {
            addr_store
                .delete_object(&principal, &addr_id, id, false)
                .await
                .unwrap();
        }

        // Check that sync token gets incremented after every operation
        assert_eq!(
            addr_store
                .get_addressbook(&principal, &addr_id, false)
                .await
                .unwrap()
                .synctoken,
            synctoken_after
        );

        for synctoken in 0..=synctoken_after {
            let (added, removed, newtoken) = addr_store
                .sync_changes(&principal, &addr_id, synctoken)
                .await
                .unwrap();

            let total_ids: Vec<&str> = added
                .iter()
                .map(|(id, _)| id.as_str())
                .chain(removed.iter().map(String::as_str))
                .collect();
            let unique_ids: HashSet<&str> = total_ids.clone().into_iter().collect();
            assert_eq!(
                total_ids.len(),
                unique_ids.len(),
                "Entries must all be unique"
            );
            assert_eq!(
                newtoken, synctoken_after,
                "since we're not doing syncs in batches the new synctoken must match the collection's synctoken"
            );
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_sync_squash_operations(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let addr_store = context.await.addr_store;

    let principal = "user".to_string();
    let addr_id = "addr".to_string();

    let addressbook = Addressbook {
        id: addr_id.clone(),
        principal: principal.clone(),
        displayname: None,
        description: None,
        deleted_at: None,
        synctoken: 0,
        push_topic: "alskdj".to_string(),
    };
    addr_store.insert_addressbook(addressbook).await.unwrap();

    let object = AddressObject::example_minimal();
    let id = "53fe5a9f-df12-478c-9ed2-1a01d83e4975".to_string();

    assert_eq!(
        (vec![], vec![], 0),
        addr_store
            .sync_changes(&principal, &addr_id, 0)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );

    addr_store
        .put_object(&principal, &addr_id, &id, object.clone(), true)
        .await
        .unwrap();

    assert_eq!(
        (vec![id.clone()], vec![], 1),
        addr_store
            .sync_changes(&principal, &addr_id, 0)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );

    addr_store
        .delete_object(&principal, &addr_id, &id, true)
        .await
        .unwrap();

    assert_eq!(
        (vec![], vec![id.clone()], 2),
        addr_store
            .sync_changes(&principal, &addr_id, 0)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );

    addr_store
        .put_object(&principal, &addr_id, &id, object.clone(), true)
        .await
        .unwrap();

    assert_eq!(
        (vec![id.clone()], vec![], 3),
        addr_store
            .sync_changes(&principal, &addr_id, 0)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );
}

#[rstest]
#[tokio::test]
async fn test_sync_no_changes_token(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let addr_store = context.await.addr_store;

    let principal = "user".to_string();
    let addr_id = "addr".to_string();

    let addressbook = Addressbook {
        id: addr_id.clone(),
        principal: principal.clone(),
        displayname: None,
        description: None,
        deleted_at: None,
        synctoken: 0,
        push_topic: "alskdj".to_string(),
    };
    addr_store.insert_addressbook(addressbook).await.unwrap();

    let object = AddressObject::example_minimal();
    let id = "53fe5a9f-df12-478c-9ed2-1a01d83e4975".to_string();

    assert_eq!(
        (vec![], vec![], 0),
        addr_store
            .sync_changes(&principal, &addr_id, 0)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );

    addr_store
        .put_object(&principal, &addr_id, &id, object.clone(), true)
        .await
        .unwrap();

    assert_eq!(
        (vec![id.clone()], vec![], 1),
        addr_store
            .sync_changes(&principal, &addr_id, 0)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );

    assert_eq!(
        (vec![], vec![], 1),
        addr_store
            .sync_changes(&principal, &addr_id, 1)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );
}

#[rstest]
#[tokio::test]
async fn test_hard_delete_scoped_to_principal(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let ctx = context.await;
    ctx.principal_store
        .insert_principal(
            Principal {
                id: "other".to_owned(),
                displayname: None,
                memberships: vec![],
                password: None,
                principal_type: PrincipalType::Individual,
            },
            false,
        )
        .await
        .unwrap();

    let object = AddressObject::example_minimal();
    for (principal, topic) in [("user", "topic-user"), ("other", "topic-other")] {
        ctx.addr_store
            .insert_addressbook(Addressbook {
                id: "addr".into(),
                principal: principal.into(),
                displayname: None,
                description: None,
                deleted_at: None,
                synctoken: 0,
                push_topic: topic.into(),
            })
            .await
            .unwrap();
        ctx.addr_store
            .put_object(principal, "addr", "obj", object.clone(), false)
            .await
            .unwrap();
    }

    ctx.addr_store
        .delete_object("other", "addr", "obj", false)
        .await
        .unwrap();

    ctx.addr_store
        .get_object("user", "addr", "obj", false)
        .await
        .unwrap();
    assert!(
        ctx.addr_store
            .get_object("other", "addr", "obj", true)
            .await
            .unwrap_err()
            .is_not_found()
    );
}

#[rstest]
#[tokio::test]
async fn test_import_creates_birthday_calendar(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let addr_store = context.await.addr_store;
    addr_store
        .import_addressbook(
            Addressbook {
                id: "imported".into(),
                principal: "user".into(),
                displayname: None,
                description: None,
                deleted_at: None,
                synctoken: 0,
                push_topic: "import-topic".into(),
            },
            vec![],
            false,
        )
        .await
        .unwrap();
    rustical_store::CalendarReadStore::get_calendar(
        &addr_store,
        "user",
        "_birthdays_imported",
        false,
    )
    .await
    .unwrap();
}

#[rstest]
#[tokio::test]
async fn test_sync_changes_skip_broken(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let ctx = context.await;
    ctx.addr_store
        .insert_addressbook(Addressbook {
            id: "addr".into(),
            principal: "user".into(),
            displayname: None,
            description: None,
            deleted_at: None,
            synctoken: 0,
            push_topic: "skip".into(),
        })
        .await
        .unwrap();
    ctx.addr_store
        .put_object(
            "user",
            "addr",
            "obj",
            AddressObject::example_minimal(),
            false,
        )
        .await
        .unwrap();
    sqlx::query("UPDATE addressobjects SET vcf = 'nope' WHERE id = $1")
        .bind("obj")
        .execute(&ctx.db)
        .await
        .unwrap();

    assert!(matches!(
        ctx.addr_store.sync_changes("user", "addr", 0).await,
        Err(rustical_store::Error::IcalError(_))
    ));

    let (send, _recv) = tokio::sync::mpsc::channel(1);
    let skipping = PostgresAddressbookStore::new(ctx.db.clone(), send, true);
    let (added, removed, _) = skipping.sync_changes("user", "addr", 0).await.unwrap();
    assert!(added.is_empty() && removed.is_empty());
}
