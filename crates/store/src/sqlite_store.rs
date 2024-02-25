use anyhow::Result;
use async_trait::async_trait;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};

use crate::{
    calendar::{Calendar, CalendarStore},
    event::Event,
};

#[derive(Debug)]
pub struct SqliteCalendarStore {
    db: SqlitePool,
}

impl SqliteCalendarStore {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[derive(Debug, Clone)]
struct EventRow {
    uid: String,
    ics: String,
}

impl TryFrom<EventRow> for Event {
    type Error = anyhow::Error;

    fn try_from(value: EventRow) -> Result<Self> {
        Event::from_ics(value.uid, value.ics)
    }
}

#[async_trait]
impl CalendarStore for SqliteCalendarStore {
    async fn get_calendar(&self, id: &str) -> Result<Calendar> {
        let cal = sqlx::query_as!(
            Calendar,
            "SELECT id, name, owner, description, color, timezone FROM calendars WHERE id = ?",
            id
        )
        .fetch_one(&self.db)
        .await?;
        Ok(cal)
    }

    async fn get_calendars(&self, _owner: &str) -> Result<Vec<Calendar>> {
        let cals = sqlx::query_as!(
            Calendar,
            "SELECT id, name, owner, description, color, timezone FROM calendars"
        )
        .fetch_all(&self.db)
        .await?;
        Ok(cals)
    }

    async fn insert_calendar(&mut self, cid: String, calendar: Calendar) -> Result<()> {
        // TODO: :(
        let name = calendar.name;
        let description = calendar.description;
        let owner = calendar.owner;
        let color = calendar.color;
        let timezone = calendar.timezone;
        sqlx::query!("INSERT INTO calendars (id, name, description, owner, color, timezone) VALUES (?, ?, ?, ?, ?, ?)", cid, name, description, owner, color, timezone).execute(&self.db).await?;
        Ok(())
    }

    async fn get_events(&self, cid: &str) -> Result<Vec<Event>> {
        let events = sqlx::query_as!(EventRow, "SELECT uid, ics FROM events WHERE cid = ?", cid)
            .fetch_all(&self.db)
            .await?
            .iter_mut()
            // TODO: this is an ugly bodge :(
            .filter_map(|row| row.clone().try_into().ok())
            .collect();
        Ok(events)
    }

    async fn get_event(&self, cid: &str, uid: &str) -> Result<Event> {
        let event = sqlx::query_as!(
            EventRow,
            "SELECT uid, ics FROM events where cid = ? AND uid = ?",
            cid,
            uid
        )
        .fetch_one(&self.db)
        .await?
        .try_into()?;
        Ok(event)
    }

    async fn upsert_event(&mut self, cid: String, uid: String, ics: String) -> Result<()> {
        // Do this extra step to ensure that the input is actually valid
        let _ = Event::from_ics(uid.to_owned(), ics.to_owned())?;
        sqlx::query!(
            "INSERT INTO events (cid, uid, ics) VALUES (?, ?, ?)",
            cid,
            uid,
            ics,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn delete_event(&mut self, cid: &str, uid: &str) -> Result<()> {
        sqlx::query!("DELETE FROM events WHERE cid = ? AND uid = ?", cid, uid)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

pub async fn create_db_pool(db_url: &str, migrate: bool) -> anyhow::Result<Pool<Sqlite>>{
    let db = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(db_url)
            .create_if_missing(true),
    )
    .await?;
    if migrate {
        println!("Running database migrations");
        sqlx::migrate!("./migrations").run(&db).await?;
    }
    Ok(db)
}