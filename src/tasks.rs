use core::future::Future;
use std::sync::Arc;

use crate::config::MaintenanceCalendarTrashbinConfig;
use rustical_store::CalendarStore;

pub async fn cleanup_trashed_calendar_entities(
    cal_store: Arc<dyn CalendarStore>,
    config: MaintenanceCalendarTrashbinConfig,
    shutdown_signal: impl Future + Send + 'static,
) {
    async fn delete_trashed_calendar_entities(
        time: chrono::NaiveDate,
        cal_store: &dyn CalendarStore,
        config: &MaintenanceCalendarTrashbinConfig,
    ) {
        if let Some(calendar_days_life) = config.deleted_calendar_lifetime {
            let limit = time - chrono::Duration::days(calendar_days_life.get().into());
            if let Err(error) = cal_store.delete_trashed_calendar_until(limit).await {
                tracing::error!(
                    ?error,
                    "Maintenance cleanup of calendar trashbin failed: {}",
                    error
                );
            }
        }

        if let Some(object_days_life) = config.deleted_object_lifetime {
            let limit = time - chrono::Duration::days(object_days_life.get().into());
            if let Err(error) = cal_store.delete_trashed_objects_until(limit).await {
                tracing::error!(
                    ?error,
                    "Maintenance cleanup of object trashbin failed: {}",
                    error
                );
            }
        }
    }

    //Perform initial cleanup in case user doesn't let server to run for more than 24 hours
    delete_trashed_calendar_entities(chrono::Utc::now().naive_utc().date(), &*cal_store, &config)
        .await;

    let mut shutdown_signal = core::pin::pin!(shutdown_signal);
    //Deletion are unlikely to be frequent so that check it daily
    let mut interval = tokio::time::interval(tokio::time::Duration::from_hours(24));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                delete_trashed_calendar_entities(chrono::Utc::now().naive_utc().date(), &*cal_store, &config).await;
            }
            _ = &mut shutdown_signal => {
                break;
            }
        }
    }
}
