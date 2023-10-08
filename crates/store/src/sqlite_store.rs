use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::calendar::{Calendar, CalendarStore};

#[derive(Debug)]
pub struct SqliteCalendarStore {
    db: Arc<SqlitePool>,
}

impl SqliteCalendarStore {
    pub fn new(db: Arc<SqlitePool>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CalendarStore for SqliteCalendarStore {
    async fn get_calendar(&self, id: &str) -> Result<Calendar> {
        let a = sqlx::query_as!(
            Calendar,
            "SELECT id, name, owner, description, color, timezone FROM calendars WHERE id = ?",
            id
        );
        Err(anyhow!("ok wow"))
    }

    async fn get_calendars(&self, _owner: &str) -> Result<Vec<Calendar>> {
        Err(anyhow!("ok wow"))
    }

    async fn insert_calendar(&mut self, _cid: String, _calendar: Calendar) -> Result<()> {
        Err(anyhow!("ok wow"))
    }

    async fn get_events(&self, _cid: &str) -> Result<Vec<crate::event::Event>> {
        Err(anyhow!("ok wow"))
    }

    async fn get_event(&self, _cid: &str, _uid: &str) -> Result<crate::event::Event> {
        Err(anyhow!("ok wow"))
    }

    async fn upsert_event(&mut self, _cid: String, _uid: String, _ics: String) -> Result<()> {
        Err(anyhow!("ok wow"))
    }

    async fn delete_event(&mut self, _cid: &str, _uid: &str) -> Result<()> {
        Err(anyhow!("ok wow"))
    }
}
