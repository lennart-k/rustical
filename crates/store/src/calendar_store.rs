use crate::{Calendar, CollectionMetadata, error::Error};
use async_trait::async_trait;
use chrono::NaiveDate;
use rustical_ical::CalendarObject;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct CalendarQuery {
    pub time_start: Option<NaiveDate>,
    pub time_end: Option<NaiveDate>,
}

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    async fn get_calendar(
        &self,
        principal: &str,
        id: &str,
        show_deleted: bool,
    ) -> Result<Calendar, Error>;
    async fn get_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error>;
    async fn get_deleted_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error>;

    async fn update_calendar(
        &self,
        principal: String,
        id: String,
        calendar: Calendar,
    ) -> Result<(), Error>;
    async fn insert_calendar(&self, calendar: Calendar) -> Result<(), Error>;
    async fn delete_calendar(
        &self,
        principal: &str,
        name: &str,
        use_trashbin: bool,
    ) -> Result<(), Error>;
    async fn restore_calendar(&self, principal: &str, name: &str) -> Result<(), Error>;
    async fn import_calendar(
        &self,
        calendar: Calendar,
        objects: Vec<CalendarObject>,
        merge_existing: bool,
    ) -> Result<(), Error>;

    async fn sync_changes(
        &self,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<CalendarObject>, Vec<String>, i64), Error>;

    /// Since the <calendar-query> rules are rather complex this function
    /// is only meant to do some prefiltering
    async fn calendar_query(
        &self,
        principal: &str,
        cal_id: &str,
        _query: CalendarQuery,
    ) -> Result<Vec<CalendarObject>, Error> {
        self.get_objects(principal, cal_id).await
    }

    async fn calendar_metadata(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<CollectionMetadata, Error>;

    async fn get_objects(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<Vec<CalendarObject>, Error>;
    async fn get_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        show_deleted: bool,
    ) -> Result<CalendarObject, Error>;
    async fn put_object(
        &self,
        principal: String,
        cal_id: String,
        object: CalendarObject,
        overwrite: bool,
    ) -> Result<(), Error>;
    async fn delete_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error>;
    async fn restore_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<(), Error>;

    // read_only refers to objects, metadata may still be updated
    fn is_read_only(&self, cal_id: &str) -> bool;
}
