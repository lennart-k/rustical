use anyhow::Result;
use async_trait::async_trait;

use crate::{calendar::Calendar, event::Event};

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    async fn get_calendar(&self, id: &str) -> Result<Calendar>;
    async fn get_calendars(&self, owner: &str) -> Result<Vec<Calendar>>;
    async fn insert_calendar(&mut self, cid: String, calendar: Calendar) -> Result<()>;
    async fn delete_calendar(&mut self, cid: &str) -> Result<()>;

    async fn get_events(&self, cid: &str) -> Result<Vec<Event>>;
    async fn get_event(&self, cid: &str, uid: &str) -> Result<Event>;
    async fn upsert_event(&mut self, cid: String, uid: String, ics: String) -> Result<()>;
    async fn delete_event(&mut self, cid: &str, uid: &str) -> Result<()>;
}
