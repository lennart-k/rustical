use crate::calendar::{Calendar, CalendarObject};
use crate::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    async fn get_calendar(&self, principal: &str, id: &str) -> Result<Calendar, Error>;
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

    async fn sync_changes(
        &self,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<CalendarObject>, Vec<String>, i64), Error>;

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
}
