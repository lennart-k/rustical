use crate::model::object::CalendarObject;
use crate::model::Calendar;
use crate::{CalendarStore, Error};
use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use sqlx::Transaction;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};
use tracing::instrument;

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
    id: String,
    ics: String,
}

impl TryFrom<CalendarObjectRow> for CalendarObject {
    type Error = Error;

    fn try_from(value: CalendarObjectRow) -> Result<Self, Error> {
        CalendarObject::from_ics(value.id, value.ics)
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
async fn log_object_operation(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &str,
    cal_id: &str,
    object_id: &str,
    operation: CalendarChangeOperation,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE calendars
        SET synctoken = synctoken + 1
        WHERE (principal, id) = (?1, ?2)"#,
        principal,
        cal_id
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO calendarobjectchangelog (principal, cal_id, object_id, operation, synctoken)
        VALUES (?1, ?2, ?3, ?4, (
            SELECT synctoken FROM calendars WHERE (principal, id) = (?1, ?2)
        ))"#,
        principal,
        cal_id,
        object_id,
        operation
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[async_trait]
impl CalendarStore for SqliteCalendarStore {
    #[instrument]
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

    #[instrument]
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

    #[instrument]
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

    #[instrument]
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
    #[instrument]
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

    #[instrument]
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

    #[instrument]
    async fn get_objects(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<Vec<CalendarObject>, Error> {
        sqlx::query_as!(
            CalendarObjectRow,
            "SELECT id, ics FROM calendarobjects WHERE principal = ? AND cal_id = ? AND deleted_at IS NULL",
            principal,
            cal_id
        )
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|row| row.try_into())
        .collect()
    }

    #[instrument]
    async fn get_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<CalendarObject, Error> {
        Ok(sqlx::query_as!(
            CalendarObjectRow,
            "SELECT id, ics FROM calendarobjects WHERE (principal, cal_id, id) = (?, ?, ?)",
            principal,
            cal_id,
            object_id
        )
        .fetch_one(&self.db)
        .await?
        .try_into()?)
    }

    #[instrument]
    async fn put_object(
        &mut self,
        principal: String,
        cal_id: String,
        object: CalendarObject,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        let (object_id, ics) = (object.get_id(), object.get_ics());

        sqlx::query!(
            "REPLACE INTO calendarobjects (principal, cal_id, id, ics) VALUES (?, ?, ?, ?)",
            principal,
            cal_id,
            object_id,
            ics
        )
        .execute(&mut *tx)
        .await?;

        log_object_operation(
            &mut tx,
            &principal,
            &cal_id,
            object_id,
            CalendarChangeOperation::Add,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    #[instrument]
    async fn delete_object(
        &mut self,
        principal: &str,
        cal_id: &str,
        id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        match use_trashbin {
            true => {
                sqlx::query!(
                    "UPDATE calendarobjects SET deleted_at = datetime(), updated_at = datetime() WHERE (principal, cal_id, id) = (?, ?, ?)",
                    principal,
                    cal_id,
                    id
                )
                .execute(&mut *tx)
                .await?;
            }
            false => {
                sqlx::query!(
                    "DELETE FROM calendarobjects WHERE cal_id = ? AND id = ?",
                    cal_id,
                    id
                )
                .execute(&mut *tx)
                .await?;
            }
        };
        log_object_operation(
            &mut tx,
            principal,
            cal_id,
            id,
            CalendarChangeOperation::Delete,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    #[instrument]
    async fn restore_object(
        &mut self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        sqlx::query!(
            r#"UPDATE calendarobjects SET deleted_at = NULL, updated_at = datetime() WHERE (principal, cal_id, id) = (?, ?, ?)"#,
            principal,
            cal_id,
            object_id
        )
        .execute(&mut *tx)
        .await?;

        log_object_operation(
            &mut tx,
            principal,
            cal_id,
            object_id,
            CalendarChangeOperation::Delete,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    #[instrument]
    async fn sync_changes(
        &self,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<CalendarObject>, Vec<String>, i64), Error> {
        struct Row {
            object_id: String,
            synctoken: i64,
        }
        let changes = sqlx::query_as!(
            Row,
            r#"
                SELECT DISTINCT object_id, max(0, synctoken) as "synctoken!: i64" from calendarobjectchangelog
                WHERE synctoken > ?
                ORDER BY synctoken ASC
            "#,
            synctoken
        )
        .fetch_all(&self.db)
        .await?;

        let mut objects = vec![];
        let mut deleted_objects = vec![];

        let new_synctoken = changes
            .last()
            .map(|&Row { synctoken, .. }| synctoken)
            .unwrap_or(0);

        for Row { object_id, .. } in changes {
            match self.get_object(principal, cal_id, &object_id).await {
                Ok(object) => objects.push(object),
                Err(Error::NotFound) => deleted_objects.push(object_id),
                Err(err) => return Err(err),
            }
        }

        Ok((objects, deleted_objects, new_synctoken))
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
