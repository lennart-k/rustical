use anyhow::Result;
use async_trait::async_trait;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};

use crate::event::Event;
use crate::{calendar::Calendar, CalendarStore, Error};

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
    type Error = Error;

    fn try_from(value: EventRow) -> Result<Self, Error> {
        Event::from_ics(value.uid, value.ics)
    }
}

#[async_trait]
impl CalendarStore for SqliteCalendarStore {
    async fn get_calendar(&self, id: &str) -> Result<Calendar, Error> {
        let cal = sqlx::query_as!(
            Calendar,
            r#"SELECT id, name, owner, "order", description, color, timezone FROM calendars WHERE id = ?"#,
            id
        )
        .fetch_one(&self.db)
        .await?;
        Ok(cal)
    }

    async fn get_calendars(&self, _owner: &str) -> Result<Vec<Calendar>, Error> {
        let cals = sqlx::query_as!(
            Calendar,
            r#"SELECT id, name, owner, "order", description, color, timezone FROM calendars"#,
        )
        .fetch_all(&self.db)
        .await?;
        Ok(cals)
    }

    async fn insert_calendar(&mut self, cid: String, calendar: Calendar) -> Result<(), Error> {
        sqlx::query!(
            r#"INSERT INTO calendars (id, name, description, owner, "order", color, timezone) VALUES (?, ?, ?, ?, ?, ?, ?)"#,
            cid,
            calendar.name,
            calendar.description,
            calendar.owner,
            calendar.order,
            calendar.color,
            calendar.timezone
        ).execute(&self.db).await?;
        Ok(())
    }

    async fn update_calendar(&mut self, cid: String, calendar: Calendar) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE calendars SET name = ?, description = ?, owner = ?, "order" = ?, color = ?, timezone = ? WHERE id = ?"#,
            calendar.name,
            calendar.description,
            calendar.owner,
            calendar.order,
            calendar.color,
            calendar.timezone,
            cid,
        ).execute(&self.db).await?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    async fn delete_calendar(&mut self, cid: &str) -> Result<(), Error> {
        sqlx::query!("DELETE FROM calendars WHERE id = ?", cid)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn get_events(&self, cid: &str) -> Result<Vec<Event>, Error> {
        sqlx::query_as!(EventRow, "SELECT uid, ics FROM events WHERE cid = ?", cid)
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|row| row.try_into())
            .collect()
    }

    async fn get_event(&self, cid: &str, uid: &str) -> Result<Event, Error> {
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

    async fn put_event(&mut self, cid: String, uid: String, ics: String) -> Result<(), Error> {
        let _ = Event::from_ics(uid.to_owned(), ics.to_owned())?;
        sqlx::query!(
            "REPLACE INTO events (cid, uid, ics) VALUES (?, ?, ?)",
            cid,
            uid,
            ics,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn delete_event(&mut self, cid: &str, uid: &str) -> Result<(), Error> {
        sqlx::query!("DELETE FROM events WHERE cid = ? AND uid = ?", cid, uid)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

pub async fn create_db_pool(db_url: &str, migrate: bool) -> anyhow::Result<Pool<Sqlite>> {
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

pub async fn create_test_store() -> anyhow::Result<SqliteCalendarStore> {
    let db = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    Ok(SqliteCalendarStore::new(db))
}
