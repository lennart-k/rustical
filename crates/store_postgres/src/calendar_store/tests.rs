use std::collections::HashSet;

use rstest::rstest;
use rustical_ical::{CalendarObject, CalendarObjectType};
use rustical_store::{
    Calendar, CalendarMetadata, CalendarReadStore, CalendarWriteStore,
    auth::{AuthenticationProvider, Principal, PrincipalType},
};

use crate::tests::{TestStoreContext, test_store_context};

#[rstest]
#[tokio::test]
async fn test_sync_full(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let cal_store = context.await.cal_store;

    let principal = "user".to_string();
    let cal_id = "cal".to_string();

    let calendar = Calendar {
        id: cal_id.clone(),
        principal: principal.clone(),
        timezone_id: None,
        meta: CalendarMetadata {
            description: None,
            order: 0,
            color: None,
            displayname: None,
        },
        deleted_at: None,
        synctoken: 0,
        push_topic: "alskdj".to_string(),
        components: vec![CalendarObjectType::Event],
        subscription_url: None,
    };
    cal_store.insert_calendar(calendar).await.unwrap();

    let object = CalendarObject::example_1();
    let id1 = "53fe5a9f-df12-478c-9ed2-1a01d83e4975";
    let id2 = "d3008843-1204-4b7e-98b8-f038328431fe";
    let id3 = "688a2ebf-fe3b-40f1-a67b-1a137552a831";
    let id4 = "61022a0d-b478-4116-98bc-a06da19dc568";

    // A list of operations where objects get removed that never existed and objects get recreated
    // https://github.com/lennart-k/rustical/issues/251
    let operations = [
        (true, id1, 1),
        (true, id2, 3),
        (false, id1, 4),
        (true, id1, 6),
        (true, id3, 8),
        (false, id4, 9),
    ];

    for (add, id, synctoken_after) in operations {
        if add {
            cal_store
                .put_object(&principal, &cal_id, id, object.clone(), true)
                .await
                .unwrap();
        } else {
            cal_store
                .delete_object(&principal, &cal_id, id, false)
                .await
                .unwrap();
        }

        // Replacing a same-UID href records both its deletion and addition.
        assert_eq!(
            cal_store
                .get_calendar(&principal, &cal_id, false)
                .await
                .unwrap()
                .synctoken,
            synctoken_after
        );

        for synctoken in 0..=synctoken_after {
            let (added, removed, newtoken) = cal_store
                .sync_changes(&principal, &cal_id, synctoken)
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
    let cal_store = context.await.cal_store;

    let principal = "user".to_string();
    let cal_id = "cal".to_string();

    let calendar = Calendar {
        id: cal_id.clone(),
        principal: principal.clone(),
        timezone_id: None,
        meta: CalendarMetadata {
            description: None,
            order: 0,
            color: None,
            displayname: None,
        },
        deleted_at: None,
        synctoken: 0,
        push_topic: "alskdj".to_string(),
        components: vec![CalendarObjectType::Event],
        subscription_url: None,
    };
    cal_store.insert_calendar(calendar).await.unwrap();

    let object = CalendarObject::example_1();
    let id = "53fe5a9f-df12-478c-9ed2-1a01d83e4975".to_string();

    assert_eq!(
        (vec![], vec![], 0),
        cal_store
            .sync_changes(&principal, &cal_id, 0)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );

    cal_store
        .put_object(&principal, &cal_id, &id, object.clone(), true)
        .await
        .unwrap();

    assert_eq!(
        (vec![id.clone()], vec![], 1),
        cal_store
            .sync_changes(&principal, &cal_id, 0)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );

    cal_store
        .delete_object(&principal, &cal_id, &id, true)
        .await
        .unwrap();

    assert_eq!(
        (vec![], vec![id.clone()], 2),
        cal_store
            .sync_changes(&principal, &cal_id, 0)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );

    cal_store
        .put_object(&principal, &cal_id, &id, object.clone(), true)
        .await
        .unwrap();

    assert_eq!(
        (vec![id.clone()], vec![], 3),
        cal_store
            .sync_changes(&principal, &cal_id, 0)
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
    let cal_store = context.await.cal_store;

    let principal = "user".to_string();
    let cal_id = "cal".to_string();

    let calendar = Calendar {
        id: cal_id.clone(),
        principal: principal.clone(),
        timezone_id: None,
        meta: CalendarMetadata {
            description: None,
            order: 0,
            color: None,
            displayname: None,
        },
        deleted_at: None,
        synctoken: 0,
        push_topic: "alskdj".to_string(),
        components: vec![CalendarObjectType::Event],
        subscription_url: None,
    };
    cal_store.insert_calendar(calendar).await.unwrap();

    let object = CalendarObject::example_1();
    let id = "53fe5a9f-df12-478c-9ed2-1a01d83e4975".to_string();

    assert_eq!(
        (vec![], vec![], 0),
        cal_store
            .sync_changes(&principal, &cal_id, 0)
            .await
            .map(|(added, deleted, token)| (
                added.into_iter().map(|(id, _)| id).collect(),
                deleted,
                token
            ))
            .unwrap(),
    );

    cal_store
        .put_object(&principal, &cal_id, &id, object.clone(), true)
        .await
        .unwrap();

    assert_eq!(
        (vec![id.clone()], vec![], 1),
        cal_store
            .sync_changes(&principal, &cal_id, 0)
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
        cal_store
            .sync_changes(&principal, &cal_id, 1)
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

    let object = CalendarObject::example_1();
    for (principal, topic) in [("user", "topic-user"), ("other", "topic-other")] {
        ctx.cal_store
            .insert_calendar(Calendar {
                id: "cal".into(),
                principal: principal.into(),
                timezone_id: None,
                meta: CalendarMetadata::default(),
                deleted_at: None,
                synctoken: 0,
                push_topic: topic.into(),
                components: vec![CalendarObjectType::Event],
                subscription_url: None,
            })
            .await
            .unwrap();
        ctx.cal_store
            .put_object(principal, "cal", "obj", object.clone(), false)
            .await
            .unwrap();
    }

    ctx.cal_store
        .delete_object("other", "cal", "obj", false)
        .await
        .unwrap();

    ctx.cal_store
        .get_object("user", "cal", "obj", false)
        .await
        .unwrap();
    assert!(
        ctx.cal_store
            .get_object("other", "cal", "obj", true)
            .await
            .unwrap_err()
            .is_not_found()
    );
}

#[rstest]
#[tokio::test]
async fn test_repair_invalid_version_is_case_insensitive(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
) {
    let ctx = context.await;
    ctx.cal_store
        .insert_calendar(Calendar {
            id: "cal".into(),
            principal: "user".into(),
            timezone_id: None,
            meta: CalendarMetadata::default(),
            deleted_at: None,
            synctoken: 0,
            push_topic: "topic".into(),
            components: vec![CalendarObjectType::Event],
            subscription_url: None,
        })
        .await
        .unwrap();
    ctx.cal_store
        .put_object("user", "cal", "object", CalendarObject::example_1(), false)
        .await
        .unwrap();
    sqlx::query("UPDATE calendarobjects SET ics = replace(ics, 'VERSION:2.0', 'version:4.0')")
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cal_store.repair_invalid_version_4_0().await.unwrap();

    let ics: String = sqlx::query_scalar(
        "SELECT ics FROM calendarobjects WHERE (principal, cal_id, id) = ($1, $2, $3)",
    )
    .bind("user")
    .bind("cal")
    .bind("object")
    .fetch_one(&ctx.db)
    .await
    .unwrap();
    assert!(ics.contains("VERSION:2.0"));
    assert!(!ics.contains("version:4.0"));
}
