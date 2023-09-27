use crate::event::Event;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Calendar {
    pub id: String,
    pub name: Option<String>,
    pub owner: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub timezone: Option<String>,
}

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    async fn get_calendar(&self, id: &str) -> Result<Calendar>;
    async fn get_calendars(&self, owner: &str) -> Result<Vec<Calendar>>;
    async fn insert_calendar(&mut self, cid: String, calendar: Calendar) -> Result<()>;

    async fn get_events(&self, cid: &str) -> Result<Vec<Event>>;
    async fn get_event(&self, cid: &str, uid: &str) -> Result<Event>;
    async fn upsert_event(&mut self, cid: String, uid: String, ics: String) -> Result<()>;
    async fn delete_event(&mut self, cid: &str, uid: &str) -> Result<()>;
}
