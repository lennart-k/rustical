use super::{ChangeOperation, SqliteStore};
use async_trait::async_trait;
use rustical_store::{Calendar, CalendarObject, CalendarStore, Error};
use sqlx::Sqlite;
use sqlx::Transaction;
use tracing::instrument;

#[derive(Debug, Clone)]
struct CalendarObjectRow {
    id: String,
    ics: String,
}

impl TryFrom<CalendarObjectRow> for CalendarObject {
    type Error = rustical_store::Error;

    fn try_from(value: CalendarObjectRow) -> Result<Self, Self::Error> {
        CalendarObject::from_ics(value.id, value.ics)
    }
}

// Logs an operation to the events
async fn log_object_operation(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &str,
    cal_id: &str,
    object_id: &str,
    operation: ChangeOperation,
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
    .await
    .map_err(crate::Error::from)?;

    sqlx::query!(
        r#"
        INSERT INTO calendarobjectchangelog (principal, cal_id, object_id, "operation", synctoken)
        VALUES (?1, ?2, ?3, ?4, (
            SELECT synctoken FROM calendars WHERE (principal, id) = (?1, ?2)
        ))"#,
        principal,
        cal_id,
        object_id,
        operation
    )
    .execute(&mut **tx)
    .await
    .map_err(crate::Error::from)?;
    Ok(())
}

#[async_trait]
impl CalendarStore for SqliteStore {
    #[instrument]
    async fn get_calendar(&self, principal: &str, id: &str) -> Result<Calendar, Error> {
        let cal = sqlx::query_as!(
            Calendar,
            r#"SELECT principal, id, synctoken, "order", displayname, description, color, timezone, timezone_id, deleted_at, subscription_url
                FROM calendars
                WHERE (principal, id) = (?, ?)"#,
            principal,
            id
        )
        .fetch_one(&self.db)
        .await.map_err(crate::Error::from)?;
        Ok(cal)
    }

    #[instrument]
    async fn get_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        let cals = sqlx::query_as!(
            Calendar,
            r#"SELECT principal, id, synctoken, displayname, "order", description, color, timezone, timezone_id, deleted_at, subscription_url
                FROM calendars
                WHERE principal = ? AND deleted_at IS NULL"#,
            principal
        )
        .fetch_all(&self.db)
        .await.map_err(crate::Error::from)?;
        Ok(cals)
    }

    #[instrument]
    async fn get_deleted_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        let cals = sqlx::query_as!(
            Calendar,
            r#"SELECT principal, id, synctoken, displayname, "order", description, color, timezone, timezone_id, deleted_at, subscription_url
                FROM calendars
                WHERE principal = ? AND deleted_at IS NOT NULL"#,
            principal
        )
        .fetch_all(&self.db)
        .await.map_err(crate::Error::from)?;
        Ok(cals)
    }

    #[instrument]
    async fn insert_calendar(&self, calendar: Calendar) -> Result<(), Error> {
        sqlx::query!(
            r#"INSERT INTO calendars (principal, id, displayname, description, "order", color, timezone, timezone_id)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
            calendar.principal,
            calendar.id,
            calendar.displayname,
            calendar.description,
            calendar.order,
            calendar.color,
            calendar.timezone,
            calendar.timezone_id
        )
        .execute(&self.db)
        .await.map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn update_calendar(
        &self,
        principal: String,
        id: String,
        calendar: Calendar,
    ) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE calendars SET principal = ?, id = ?, displayname = ?, description = ?, "order" = ?, color = ?, timezone = ?, timezone_id = ?
                WHERE (principal, id) = (?, ?)"#,
            calendar.principal,
            calendar.id,
            calendar.displayname,
            calendar.description,
            calendar.order,
            calendar.color,
            calendar.timezone,
            calendar.timezone_id,
            principal,
            id
        ).execute(&self.db).await.map_err(crate::Error::from)?;
        if result.rows_affected() == 0 {
            return Err(rustical_store::Error::NotFound);
        }
        Ok(())
    }

    // Does not actually delete the calendar but just disables it
    #[instrument]
    async fn delete_calendar(
        &self,
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
                .await.map_err(crate::Error::from)?;
            }
            false => {
                sqlx::query!(
                    r#"DELETE FROM calendars WHERE (principal, id) = (?, ?)"#,
                    principal,
                    id
                )
                .execute(&self.db)
                .await
                .map_err(crate::Error::from)?;
            }
        };
        Ok(())
    }

    #[instrument]
    async fn restore_calendar(&self, principal: &str, id: &str) -> Result<(), Error> {
        sqlx::query!(
            r"UPDATE calendars SET deleted_at = NULL WHERE (principal, id) = (?, ?)",
            principal,
            id
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
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
        .await.map_err(crate::Error::from)?
        .into_iter()
        .map(|row| row.try_into().map_err(rustical_store::Error::from))
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
        .await
        .map_err(crate::Error::from)?
        .try_into()?)
    }

    #[instrument]
    async fn put_object(
        &self,
        principal: String,
        cal_id: String,
        object: CalendarObject,
        overwrite: bool,
    ) -> Result<(), Error> {
        // TODO: Prevent objects from being commited to a subscription calendar
        let mut tx = self.db.begin().await.map_err(crate::Error::from)?;

        let (object_id, ics) = (object.get_id(), object.get_ics());

        (if overwrite {
            sqlx::query!(
                "REPLACE INTO calendarobjects (principal, cal_id, id, ics) VALUES (?, ?, ?, ?)",
                principal,
                cal_id,
                object_id,
                ics
            )
        } else {
            // If the object already exists a database error is thrown and handled in error.rs
            sqlx::query!(
                "INSERT INTO calendarobjects (principal, cal_id, id, ics) VALUES (?, ?, ?, ?)",
                principal,
                cal_id,
                object_id,
                ics,
            )
        })
        .execute(&mut *tx)
        .await
        .map_err(crate::Error::from)?;

        log_object_operation(
            &mut tx,
            &principal,
            &cal_id,
            object_id,
            ChangeOperation::Add,
        )
        .await?;

        tx.commit().await.map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn delete_object(
        &self,
        principal: &str,
        cal_id: &str,
        id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await.map_err(crate::Error::from)?;

        match use_trashbin {
            true => {
                sqlx::query!(
                    "UPDATE calendarobjects SET deleted_at = datetime(), updated_at = datetime() WHERE (principal, cal_id, id) = (?, ?, ?)",
                    principal,
                    cal_id,
                    id
                )
                .execute(&mut *tx)
                .await.map_err(crate::Error::from)?;
            }
            false => {
                sqlx::query!(
                    "DELETE FROM calendarobjects WHERE cal_id = ? AND id = ?",
                    cal_id,
                    id
                )
                .execute(&mut *tx)
                .await
                .map_err(crate::Error::from)?;
            }
        };
        log_object_operation(&mut tx, principal, cal_id, id, ChangeOperation::Delete).await?;
        tx.commit().await.map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn restore_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await.map_err(crate::Error::from)?;

        sqlx::query!(
            r#"UPDATE calendarobjects SET deleted_at = NULL, updated_at = datetime() WHERE (principal, cal_id, id) = (?, ?, ?)"#,
            principal,
            cal_id,
            object_id
        )
        .execute(&mut *tx)
        .await.map_err(crate::Error::from)?;

        log_object_operation(&mut tx, principal, cal_id, object_id, ChangeOperation::Add).await?;
        tx.commit().await.map_err(crate::Error::from)?;
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
        .await.map_err(crate::Error::from)?;

        let mut objects = vec![];
        let mut deleted_objects = vec![];

        let new_synctoken = changes
            .last()
            .map(|&Row { synctoken, .. }| synctoken)
            .unwrap_or(0);

        for Row { object_id, .. } in changes {
            match self.get_object(principal, cal_id, &object_id).await {
                Ok(object) => objects.push(object),
                Err(rustical_store::Error::NotFound) => deleted_objects.push(object_id),
                Err(err) => return Err(err),
            }
        }

        Ok((objects, deleted_objects, new_synctoken))
    }
}
