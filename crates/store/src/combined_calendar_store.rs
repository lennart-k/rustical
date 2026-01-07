use crate::{Calendar, CalendarStore, calendar_store::CalendarQuery};
use async_trait::async_trait;
use rustical_ical::CalendarObject;
use std::{collections::HashMap, sync::Arc};

pub trait PrefixedCalendarStore: CalendarStore {
    const PREFIX: &'static str;
}

#[derive(Clone)]
pub struct CombinedCalendarStore {
    stores: HashMap<&'static str, Arc<dyn CalendarStore>>,
    default: Arc<dyn CalendarStore>,
}

impl CombinedCalendarStore {
    pub fn new(default: Arc<dyn CalendarStore>) -> Self {
        Self {
            stores: HashMap::new(),
            default,
        }
    }

    #[must_use]
    pub fn with_store<CS: PrefixedCalendarStore>(mut self, store: Arc<CS>) -> Self {
        let store: Arc<dyn CalendarStore> = store;
        self.stores.insert(CS::PREFIX, store);
        self
    }

    fn store_for_id(&self, id: &str) -> Arc<dyn CalendarStore> {
        self.stores
            .iter()
            .find(|&(prefix, _store)| id.starts_with(prefix))
            .map_or_else(|| self.default.clone(), |(_prefix, store)| store.clone())
    }
}

#[async_trait]
impl CalendarStore for CombinedCalendarStore {
    #[inline]
    async fn get_calendar(
        &self,
        principal: &str,
        id: &str,
        show_deleted: bool,
    ) -> Result<crate::Calendar, crate::Error> {
        self.store_for_id(id)
            .get_calendar(principal, id, show_deleted)
            .await
    }

    async fn update_calendar(
        &self,
        principal: &str,
        id: &str,
        calendar: Calendar,
    ) -> Result<(), crate::Error> {
        self.store_for_id(id)
            .update_calendar(principal, id, calendar)
            .await
    }

    async fn insert_calendar(&self, calendar: crate::Calendar) -> Result<(), crate::Error> {
        self.store_for_id(&calendar.id)
            .insert_calendar(calendar)
            .await
    }

    async fn delete_calendar(
        &self,
        principal: &str,
        name: &str,
        use_trashbin: bool,
    ) -> Result<(), crate::Error> {
        self.store_for_id(name)
            .delete_calendar(principal, name, use_trashbin)
            .await
    }

    async fn restore_calendar(&self, principal: &str, name: &str) -> Result<(), crate::Error> {
        self.store_for_id(name)
            .restore_calendar(principal, name)
            .await
    }

    async fn sync_changes(
        &self,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<(String, CalendarObject)>, Vec<String>, i64), crate::Error> {
        self.store_for_id(cal_id)
            .sync_changes(principal, cal_id, synctoken)
            .await
    }

    async fn import_calendar(
        &self,
        calendar: crate::Calendar,
        objects: Vec<CalendarObject>,
        merge_existing: bool,
    ) -> Result<(), crate::Error> {
        self.store_for_id(&calendar.id)
            .import_calendar(calendar, objects, merge_existing)
            .await
    }

    async fn calendar_query(
        &self,
        principal: &str,
        cal_id: &str,
        query: CalendarQuery,
    ) -> Result<Vec<(String, CalendarObject)>, crate::Error> {
        self.store_for_id(cal_id)
            .calendar_query(principal, cal_id, query)
            .await
    }

    async fn restore_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<(), crate::Error> {
        self.store_for_id(cal_id)
            .restore_object(principal, cal_id, object_id)
            .await
    }

    async fn calendar_metadata(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<crate::CollectionMetadata, crate::Error> {
        self.store_for_id(cal_id)
            .calendar_metadata(principal, cal_id)
            .await
    }

    async fn get_objects(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<Vec<(String, CalendarObject)>, crate::Error> {
        self.store_for_id(cal_id)
            .get_objects(principal, cal_id)
            .await
    }

    async fn put_objects(
        &self,
        principal: &str,
        cal_id: &str,
        objects: Vec<(String, CalendarObject)>,
        overwrite: bool,
    ) -> Result<(), crate::Error> {
        self.store_for_id(cal_id)
            .put_objects(principal, cal_id, objects, overwrite)
            .await
    }

    async fn delete_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        use_trashbin: bool,
    ) -> Result<(), crate::Error> {
        self.store_for_id(cal_id)
            .delete_object(principal, cal_id, object_id, use_trashbin)
            .await
    }

    async fn get_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        show_deleted: bool,
    ) -> Result<rustical_ical::CalendarObject, crate::Error> {
        self.store_for_id(cal_id)
            .get_object(principal, cal_id, object_id, show_deleted)
            .await
    }

    async fn get_calendars(&self, principal: &str) -> Result<Vec<crate::Calendar>, crate::Error> {
        let mut calendars = self.default.get_calendars(principal).await?;
        for store in self.stores.values() {
            calendars.extend(store.get_calendars(principal).await?);
        }
        Ok(calendars)
    }

    async fn get_deleted_calendars(
        &self,
        principal: &str,
    ) -> Result<Vec<crate::Calendar>, crate::Error> {
        let mut calendars = self.default.get_deleted_calendars(principal).await?;
        for store in self.stores.values() {
            calendars.extend(store.get_deleted_calendars(principal).await?);
        }
        Ok(calendars)
    }

    fn is_read_only(&self, cal_id: &str) -> bool {
        self.store_for_id(cal_id).is_read_only(cal_id)
    }
}
