#[cfg(test)]
mod tests {
    use crate::tests::{TestStoreContext, test_store_context};
    use rstest::rstest;
    use rustical_store::{Calendar, CalendarMetadata, CalendarStore};

    #[rstest]
    #[tokio::test]
    async fn test_calendar_store(
        #[future]
        #[from(test_store_context)]
        context: TestStoreContext,
    ) {
        let TestStoreContext { cal_store, .. } = context.await;

        let cal_store = cal_store;

        let cal = Calendar {
            principal: "fake-user".to_string(),
            timezone_id: None,
            deleted_at: None,
            meta: CalendarMetadata::default(),
            id: "cal".to_string(),
            synctoken: 0,
            subscription_url: None,
            push_topic: "alskdj".to_string(),
            components: vec![],
        };

        assert!(
            cal_store.insert_calendar(cal).await.is_err(),
            "This should fail due to the user not existing "
        );

        let cal = Calendar {
            principal: "user".to_string(),
            timezone_id: None,
            deleted_at: None,
            meta: CalendarMetadata::default(),
            id: "cal".to_string(),
            synctoken: 0,
            subscription_url: None,
            push_topic: "alskdj".to_string(),
            components: vec![],
        };

        cal_store.insert_calendar(cal.clone()).await.unwrap();

        assert_eq!(
            cal_store.get_calendar("user", "cal", false).await.unwrap(),
            cal
        );

        cal_store
            .delete_calendar("user", "cal", true)
            .await
            .unwrap();

        let Err(err) = cal_store.get_calendar("user", "cal", false).await else {
            panic!()
        };
        assert!(err.is_not_found());

        cal_store.get_calendar("user", "cal", true).await.unwrap();

        cal_store.restore_calendar("user", "cal").await.unwrap();

        cal_store
            .delete_calendar("user", "cal", false)
            .await
            .unwrap();

        let Err(err) = cal_store.get_calendar("user", "cal", true).await else {
            panic!()
        };
        assert!(err.is_not_found());
    }
}
