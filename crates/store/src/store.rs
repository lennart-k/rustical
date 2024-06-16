use anyhow::Result;
use async_trait::async_trait;

use crate::error::Error;
use crate::{calendar::Calendar, event::Event};

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    async fn get_calendar(&self, id: &str) -> Result<Calendar, Error>;
    async fn get_calendars(&self, owner: &str) -> Result<Vec<Calendar>, Error>;
    async fn update_calendar(&mut self, cid: String, calendar: Calendar) -> Result<(), Error>;
    async fn insert_calendar(&mut self, cid: String, calendar: Calendar) -> Result<(), Error>;
    async fn delete_calendar(&mut self, cid: &str) -> Result<(), Error>;

    async fn get_events(&self, cid: &str) -> Result<Vec<Event>, Error>;
    async fn get_event(&self, cid: &str, uid: &str) -> Result<Event, Error>;
    async fn upsert_event(&mut self, cid: String, uid: String, ics: String) -> Result<(), Error>;
    async fn delete_event(&mut self, cid: &str, uid: &str) -> Result<(), Error>;
}
