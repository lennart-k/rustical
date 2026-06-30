#[cfg(test)]
mod tests {
    use crate::tests::{TestStoreContext, test_store_context};
    use rstest::rstest;
    use rustical_ical::CalendarObject;
    use rustical_store::{Calendar, CalendarMetadata, CalendarReadStore, CalendarWriteStore};

    const CALENDAR_OBJECT_ICS: &str = r#"
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//iCalendar Event//EN
CALSCALE:GREGORIAN
BEGIN:VEVENT
UID:20260628T153000Z-123456@domain.com
DTSTAMP:20260628T153000Z
DTSTART:20260715T100000Z
DTEND:20260715T110000Z
SUMMARY:iCal Event
DESCRIPTION:Basic calendar event.
LOCATION:Meeting Room A
END:VEVENT
END:VCALENDAR"#;

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

        let object_id = "test-object";
        let object =
            CalendarObject::from_ics(CALENDAR_OBJECT_ICS.to_owned()).expect("to parse ics");

        cal_store
            .put_object(&cal.principal, &cal.id, object_id, object.clone(), false)
            .await
            .expect("to insert object");
        cal_store
            .delete_calendar("user", "cal", true)
            .await
            .unwrap();

        let Err(err) = cal_store.get_calendar("user", "cal", false).await else {
            panic!()
        };
        assert!(err.is_not_found());

        let fetched_object = cal_store
            .get_object(&cal.principal, &cal.id, object_id, false)
            .await
            .expect("object remains");
        assert_eq!(fetched_object.get_uid(), object.get_uid());

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

        match cal_store
            .get_object(&cal.principal, &cal.id, object_id, false)
            .await
        {
            Ok(_object) => panic!("Calendar deletion should cascade to relevant objects deletion"),
            Err(error) => assert!(error.is_not_found()),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn should_deleted_trashed_calendar_andobjects_by_date_limit(
        #[future]
        #[from(test_store_context)]
        context: TestStoreContext,
    ) {
        let TestStoreContext { cal_store, .. } = context.await;

        let cal_store = cal_store;

        let cal = Calendar {
            principal: "user".to_string(),
            timezone_id: None,
            deleted_at: None,
            meta: CalendarMetadata::default(),
            id: "trashed-cal".to_string(),
            synctoken: 0,
            subscription_url: None,
            push_topic: "trashed".to_string(),
            components: vec![],
        };

        cal_store.insert_calendar(cal.clone()).await.unwrap();

        let object_id = "test-trash-object";
        let object =
            CalendarObject::from_ics(CALENDAR_OBJECT_ICS.to_owned()).expect("to parse ics");

        cal_store
            .put_object(&cal.principal, &cal.id, object_id, object.clone(), false)
            .await
            .expect("to insert object");

        let now = chrono::Utc::now().date_naive();
        //Delete object and calendar
        cal_store
            .delete_object(&cal.principal, &cal.id, object_id, true)
            .await
            .expect("to delete");
        cal_store
            .delete_calendar(&cal.principal, &cal.id, true)
            .await
            .unwrap();

        //Verify we delete only BEFORE timestamp
        cal_store.prune_deleted_objects(now).await.expect("success");
        cal_store
            .get_object(&cal.principal, &cal.id, object_id, true)
            .await
            .expect("Nothing deleted yet");
        cal_store
            .prune_deleted_calendars(now)
            .await
            .expect("success");
        cal_store
            .get_calendar(&cal.principal, &cal.id, true)
            .await
            .expect("Nothing deleted yet");

        //delete everything that was marked for deletion before tomorrow
        cal_store
            .prune_deleted_objects(now + chrono::Duration::days(1))
            .await
            .expect("success");
        let error = cal_store
            .get_object(&cal.principal, &cal.id, object_id, true)
            .await
            .expect_err("object should be deleted");
        assert!(error.is_not_found());
        cal_store
            .prune_deleted_calendars(now + chrono::Duration::days(1))
            .await
            .expect("success");
        let error = cal_store
            .get_calendar(&cal.principal, &cal.id, true)
            .await
            .expect_err("calendar should be deleted");
        assert!(error.is_not_found());
    }
}
