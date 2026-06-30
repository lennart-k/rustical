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
        let before = time - chrono::Duration::days(trash_retention_days.get().into());
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
