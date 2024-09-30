use crate::model::object::CalendarObject;
use crate::model::Calendar;
use crate::{CalendarStore, Error};
use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use sqlx::Transaction;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};

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
struct CalendarObjectRow {
    uid: String,
    ics: String,
}

impl TryFrom<CalendarObjectRow> for CalendarObject {
    type Error = Error;

    fn try_from(value: CalendarObjectRow) -> Result<Self, Error> {
        CalendarObject::from_ics(value.uid, value.ics)
    }
}

#[derive(Debug, Clone, Serialize, sqlx::Type)]
#[serde(rename_all = "kebab-case")]
enum CalendarChangeOperation {
    // There's no distinction between Add and Modify
    Add,
    Delete,
}

// Logs an operation to the events
async fn log_event_operation(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &str,
    cid: &str,
    uid: &str,
    operation: CalendarChangeOperation,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE calendars
        SET synctoken = synctoken + 1
        WHERE (principal, id) = (?1, ?2)"#,
        principal,
        cid
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO eventchangelog (principal, cid, uid, operation, synctoken)
        VALUES (?1, ?2, ?3, ?4, (
            SELECT synctoken FROM calendars WHERE (principal, id) = (?1, ?2)
        ))"#,
        principal,
        cid,
        uid,
        operation
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[async_trait]
impl CalendarStore for SqliteCalendarStore {
    async fn get_calendar(&self, principal: &str, id: &str) -> Result<Calendar, Error> {
        let cal = sqlx::query_as!(
            Calendar,
            r#"SELECT principal, id, synctoken, "order", displayname, description, color, timezone, deleted_at
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
            r#"SELECT principal, id, synctoken, displayname, "order", description, color, timezone, deleted_at
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

    async fn get_objects(&self, principal: &str, cid: &str) -> Result<Vec<CalendarObject>, Error> {
        sqlx::query_as!(
            CalendarObjectRow,
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

    async fn get_object(
        &self,
        principal: &str,
        cid: &str,
        uid: &str,
    ) -> Result<CalendarObject, Error> {
        let event = sqlx::query_as!(
            CalendarObjectRow,
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

    async fn put_object(
        &mut self,
        principal: String,
        cid: String,
        uid: String,
        ics: String,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        // input validation
        CalendarObject::from_ics(uid.to_owned(), ics.to_owned())?;
        sqlx::query!(
            "REPLACE INTO events (principal, cid, uid, ics) VALUES (?, ?, ?, ?)",
            principal,
            cid,
            uid,
            ics,
        )
        .execute(&mut *tx)
        .await?;

        log_event_operation(
            &mut tx,
            &principal,
            &cid,
            &uid,
            CalendarChangeOperation::Add,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn delete_object(
        &mut self,
        principal: &str,
        cid: &str,
        uid: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        match use_trashbin {
            true => {
                sqlx::query!(
                    "UPDATE events SET deleted_at = datetime(), updated_at = datetime() WHERE (principal, cid, uid) = (?, ?, ?)",
                    principal,
                    cid,
                    uid
                )
                .execute(&mut *tx)
                .await?;
            }
            false => {
                sqlx::query!("DELETE FROM events WHERE cid = ? AND uid = ?", cid, uid)
                    .execute(&mut *tx)
                    .await?;
            }
        };
        log_event_operation(
            &mut tx,
            principal,
            cid,
            uid,
            CalendarChangeOperation::Delete,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn restore_object(&mut self, principal: &str, cid: &str, uid: &str) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        sqlx::query!(
            r#"UPDATE events SET deleted_at = NULL, updated_at = datetime() WHERE (principal, cid, uid) = (?, ?, ?)"#,
            principal,
            cid,
            uid
        )
        .execute(&mut *tx)
        .await?;

        log_event_operation(
            &mut tx,
            principal,
            cid,
            uid,
            CalendarChangeOperation::Delete,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn sync_changes(
        &self,
        principal: &str,
        cid: &str,
        synctoken: i64,
    ) -> Result<(Vec<CalendarObject>, Vec<String>, i64), Error> {
        struct Row {
            uid: String,
            synctoken: i64,
        }
        let changes = sqlx::query_as!(
            Row,
            r#"
                SELECT DISTINCT uid, max(0, synctoken) as "synctoken!: i64" from eventchangelog
                WHERE synctoken > ?
                ORDER BY synctoken ASC
            "#,
            synctoken
        )
        .fetch_all(&self.db)
        .await?;

        let mut events = vec![];
        let mut deleted_events = vec![];

        let new_synctoken = changes
            .last()
            .map(|&Row { synctoken, .. }| synctoken)
            .unwrap_or(0);

        for Row { uid, .. } in changes {
            match self.get_object(principal, cid, &uid).await {
                Ok(event) => events.push(event),
                Err(Error::NotFound) => deleted_events.push(uid),
                Err(err) => return Err(err),
            }
        }

        Ok((events, deleted_events, new_synctoken))
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
