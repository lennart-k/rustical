use core::future::Future;
use core::num::NonZeroU32;
use std::sync::Arc;

use rustical_store::CalendarStore;

pub async fn cleanup_trashed_calendar_entities(
    cal_store: Arc<dyn CalendarStore>,
    trash_retention_days: NonZeroU32,
    shutdown_signal: impl Future + Send + 'static,
) {
    async fn delete_trashed_calendar_entities(
        time: chrono::NaiveDate,
        cal_store: &dyn CalendarStore,
        trash_retention_days: NonZeroU32,
    ) {
        //Default sub operation can panic so avoid it
        let before =
            match time.checked_sub_days(chrono::Days::new(trash_retention_days.get().into())) {
                Some(before) => before,
                None => {
                    const UTC: chrono::NaiveDate = match chrono::NaiveDate::from_epoch_days(0) {
                        Some(utc) => utc,
                        None => unreachable!(),
                    };
                    UTC
                }
            };

        if let Err(error) = cal_store.prune_deleted_calendars(before).await {
            tracing::error!(
                ?error,
                "Maintenance cleanup of calendar trashbin failed: {}",
                error
            );
        }

        if let Err(error) = cal_store.prune_deleted_objects(before).await {
            tracing::error!(
                ?error,
                "Maintenance cleanup of object trashbin failed: {}",
                error
            );
        }
    }

    // Perform initial cleanup in case user doesn't let server run for more than 24 hours
    delete_trashed_calendar_entities(
        chrono::Utc::now().naive_utc().date(),
        &*cal_store,
        trash_retention_days,
    )
    .await;

    let mut shutdown_signal = core::pin::pin!(shutdown_signal);
    // Deletion is unlikely to be frequent hence daily
    let mut interval = tokio::time::interval(tokio::time::Duration::from_hours(24));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                delete_trashed_calendar_entities(chrono::Utc::now().naive_utc().date(), &*cal_store, trash_retention_days).await;
            }
            _ = &mut shutdown_signal => {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::future;
    use core::num::NonZeroU32;
    use std::sync::Arc;

    use super::cleanup_trashed_calendar_entities;
    use rustical_store::auth::AuthenticationProvider;
    use rustical_store::{CalendarReadStore, CalendarWriteStore};
    use rustical_store_sqlite::calendar_store::SqliteCalendarStore;
    use rustical_store_sqlite::create_db_pool;
    use rustical_store_sqlite::principal_store::SqlitePrincipalStore;

    const CONFIG_RETENTION_DAYS: NonZeroU32 = match NonZeroU32::new(1) {
        Some(result) => result,
        None => unreachable!(),
    };

    #[tokio::test]
    async fn should_shutdown_cleanup_task_on_demand() {
        let db = create_db_pool("sqlite://:memory:", true)
            .await
            .expect("to create db");
        let (send, _recv) = tokio::sync::mpsc::channel(1000);
        let cal_store = Arc::new(SqliteCalendarStore::new(db.clone(), send, true));
        let principal_store = SqlitePrincipalStore::new(db);
        let principal = rustical_store::auth::Principal {
            id: "user".to_owned(),
            displayname: None,
            password: None,
            principal_type: rustical_store::auth::PrincipalType::Individual,
            memberships: Vec::new(),
        };
        principal_store
            .insert_principal(principal, false)
            .await
            .expect("to insert principal");

        let calendar = rustical_store::Calendar {
            meta: Default::default(),
            principal: "user".to_owned(),
            id: "id".to_owned(),
            ..Default::default()
        };
        cal_store
            .insert_calendar(calendar)
            .await
            .expect("insert calendar");
        cal_store
            .delete_calendar("user", "id", true)
            .await
            .expect("delete into trashbin");

        //should exit task as soon as it enters loop
        cleanup_trashed_calendar_entities(
            cal_store.clone(),
            CONFIG_RETENTION_DAYS,
            future::ready(()),
        )
        .await;

        let result = cal_store
            .get_calendar("user", "id", true)
            .await
            .expect("should not be deleted on the same day creation");
        assert_eq!(result.id, "id");
    }
}
