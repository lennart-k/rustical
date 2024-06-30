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
    async fn get_calendar(&self, principal: &str, id: &str) -> Result<Calendar, Error> {
        let cal = sqlx::query_as!(
            Calendar,
            r#"SELECT principal, id, "order", displayname, description, color, timezone, deleted_at
                FROM calendars
                WHERE (principal, id) = (?, ?)"#,
            principal,
            id
        )
        .fetch_one(&self.db)
        .await?;
        Ok(cal)
    }

    async fn get_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        let cals = sqlx::query_as!(
            Calendar,
            r#"SELECT principal, id, displayname, "order", description, color, timezone, deleted_at
                FROM calendars
                WHERE principal = ? AND deleted_at IS NULL"#,
            principal
        )
        .fetch_all(&self.db)
        .await?;
        Ok(cals)
    }

    async fn insert_calendar(&mut self, calendar: Calendar) -> Result<(), Error> {
        sqlx::query!(
            r#"INSERT INTO calendars (principal, id, displayname, description, "order", color, timezone)
                VALUES (?, ?, ?, ?, ?, ?, ?)"#,
            calendar.principal,
            calendar.id,
            calendar.displayname,
            calendar.description,
            calendar.order,
            calendar.color,
            calendar.timezone
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn update_calendar(
        &mut self,
        principal: String,
        id: String,
        calendar: Calendar,
    ) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE calendars SET principal = ?, id = ?, displayname = ?, description = ?, "order" = ?, color = ?, timezone = ?
                WHERE (principal, id) = (?, ?)"#,
            calendar.principal,
            calendar.id,
            calendar.displayname,
            calendar.description,
            calendar.order,
            calendar.color,
            calendar.timezone,
            principal,
            id
        ).execute(&self.db).await?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    // Does not actually delete the calendar but just disables it
    async fn delete_calendar(
        &mut self,
        principal: &str,
        id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        match use_trashbin {
            true => {
                sqlx::query!(
                    r#"UPDATE calendars SET deleted_at = datetime() WHERE (principal, id) = (?, ?)"#,
                    principal, id
                )
                .execute(&self.db)
                .await?;
            }
            false => {
                sqlx::query!(
                    r#"DELETE FROM calendars WHERE (principal, id) = (?, ?)"#,
                    principal,
                    id
                )
                .execute(&self.db)
                .await?;
            }
        };
        Ok(())
    }

    // Does not actually delete the calendar but just disables it
    async fn restore_calendar(&mut self, principal: &str, id: &str) -> Result<(), Error> {
        sqlx::query!(
            r"UPDATE calendars SET deleted_at = NULL WHERE (principal, id) = (?, ?)",
            principal,
            id
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn get_events(&self, principal: &str, cid: &str) -> Result<Vec<Event>, Error> {
        sqlx::query_as!(
            EventRow,
            "SELECT uid, ics FROM events WHERE principal = ? AND cid = ? AND deleted_at IS NULL",
            principal,
            cid
        )
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|row| row.try_into())
        .collect()
    }

    async fn get_event(&self, principal: &str, cid: &str, uid: &str) -> Result<Event, Error> {
        let event = sqlx::query_as!(
            EventRow,
            "SELECT uid, ics FROM events WHERE (principal, cid, uid) = (?, ?, ?)",
            principal,
            cid,
            uid
        )
        .fetch_one(&self.db)
        .await?
        .try_into()?;
        Ok(event)
    }

    async fn put_event(
        &mut self,
        principal: String,
        cid: String,
        uid: String,
        ics: String,
    ) -> Result<(), Error> {
        let _ = Event::from_ics(uid.to_owned(), ics.to_owned())?;
        sqlx::query!(
            "REPLACE INTO events (principal, cid, uid, ics) VALUES (?, ?, ?, ?)",
            principal,
            cid,
            uid,
            ics,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn delete_event(
        &mut self,
        principal: &str,
        cid: &str,
        uid: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        match use_trashbin {
            true => {
                sqlx::query!(
                    "UPDATE events SET deleted_at = datetime() WHERE (principal, cid, uid) = (?, ?, ?)",
                    principal,
                    cid,
                    uid
                )
                .execute(&self.db)
                .await?;
            }
            false => {
                sqlx::query!("DELETE FROM events WHERE cid = ? AND uid = ?", cid, uid)
                    .execute(&self.db)
                    .await?;
            }
        };
        Ok(())
    }

    async fn restore_event(&mut self, principal: &str, cid: &str, uid: &str) -> Result<(), Error> {
        sqlx::query!(
            r#"UPDATE events SET deleted_at = NULL WHERE (principal, cid, uid) = (?, ?, ?)"#,
            principal,
            cid,
            uid
        )
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
