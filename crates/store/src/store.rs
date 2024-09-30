use anyhow::Result;
use async_trait::async_trait;

use crate::error::Error;
use crate::model::object::CalendarObject;
use crate::model::Calendar;

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    async fn get_calendar(&self, principal: &str, id: &str) -> Result<Calendar, Error>;
    async fn get_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error>;

    async fn update_calendar(
        &mut self,
        principal: String,
        id: String,
        calendar: Calendar,
    ) -> Result<(), Error>;
    async fn insert_calendar(&mut self, calendar: Calendar) -> Result<(), Error>;
    async fn delete_calendar(
        &mut self,
        principal: &str,
        name: &str,
        use_trashbin: bool,
    ) -> Result<(), Error>;
    async fn restore_calendar(&mut self, principal: &str, name: &str) -> Result<(), Error>;

    async fn sync_changes(
        &self,
        principal: &str,
        cid: &str,
        synctoken: i64,
    ) -> Result<(Vec<CalendarObject>, Vec<String>, i64), Error>;

    async fn get_objects(&self, principal: &str, cid: &str) -> Result<Vec<CalendarObject>, Error>;
    async fn get_object(
        &self,
        principal: &str,
        cid: &str,
        uid: &str,
    ) -> Result<CalendarObject, Error>;
    async fn put_object(
        &mut self,
        principal: String,
        cid: String,
        uid: String,
        ics: String,
    ) -> Result<(), Error>;
    async fn delete_object(
        &mut self,
        principal: &str,
        cid: &str,
        uid: &str,
        use_trashbin: bool,
    ) -> Result<(), Error>;
    async fn restore_object(&mut self, principal: &str, cid: &str, uid: &str) -> Result<(), Error>;
}
