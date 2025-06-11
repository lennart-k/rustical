use super::ChangeOperation;
use async_trait::async_trait;
use chrono::TimeDelta;
use derive_more::derive::Constructor;
use rustical_ical::{CalDateTime, CalendarObject, CalendarObjectType};
use rustical_store::calendar_store::CalendarQuery;
use rustical_store::synctoken::format_synctoken;
use rustical_store::{Calendar, CalendarStore, Error};
use rustical_store::{CollectionOperation, CollectionOperationType};
use sqlx::types::chrono::NaiveDateTime;
use sqlx::{Acquire, Executor, Sqlite, SqlitePool, Transaction};
use tokio::sync::mpsc::Sender;
use tracing::{error, instrument};

#[derive(Debug, Clone)]
struct CalendarObjectRow {
    id: String,
    ics: String,
}

impl TryFrom<CalendarObjectRow> for CalendarObject {
    type Error = rustical_store::Error;

    fn try_from(value: CalendarObjectRow) -> Result<Self, Self::Error> {
        Ok(CalendarObject::from_ics(value.id, value.ics)?)
    }
}

#[derive(Debug, Default, Clone)]
struct CalendarRow {
    principal: String,
    id: String,
    displayname: Option<String>,
    order: i64,
    description: Option<String>,
    color: Option<String>,
    timezone: Option<String>,
    timezone_id: Option<String>,
    deleted_at: Option<NaiveDateTime>,
    synctoken: i64,
    subscription_url: Option<String>,
    push_topic: String,
    comp_event: bool,
    comp_todo: bool,
    comp_journal: bool,
}

impl From<CalendarRow> for Calendar {
    fn from(value: CalendarRow) -> Self {
        let mut components = vec![];
        if value.comp_event {
            components.push(CalendarObjectType::Event);
        }
        if value.comp_todo {
            components.push(CalendarObjectType::Todo);
        }
        if value.comp_journal {
            components.push(CalendarObjectType::Journal);
        }
        Self {
            principal: value.principal,
            id: value.id,
            displayname: value.displayname,
            order: value.order,
            description: value.description,
            color: value.color,
            timezone: value.timezone,
            timezone_id: value.timezone_id,
            deleted_at: value.deleted_at,
            synctoken: value.synctoken,
            subscription_url: value.subscription_url,
            push_topic: value.push_topic,
            components,
        }
    }
}

#[derive(Debug, Constructor)]
pub struct SqliteCalendarStore {
    db: SqlitePool,
    sender: Sender<CollectionOperation>,
}

impl SqliteCalendarStore {
    async fn _get_calendar<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        id: &str,
    ) -> Result<Calendar, Error> {
        let cal = sqlx::query_as!(
            CalendarRow,
            r#"SELECT *
                FROM calendars
                WHERE (principal, id) = (?, ?)"#,
            principal,
            id
        )
        .fetch_one(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(cal.into())
    }

    async fn _get_calendars<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
    ) -> Result<Vec<Calendar>, Error> {
        let cals = sqlx::query_as!(
            CalendarRow,
            r#"SELECT *
                FROM calendars
                WHERE principal = ? AND deleted_at IS NULL"#,
            principal
        )
        .fetch_all(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(cals.into_iter().map(Calendar::from).collect())
    }

    async fn _get_deleted_calendars<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
    ) -> Result<Vec<Calendar>, Error> {
        let cals = sqlx::query_as!(
            CalendarRow,
            r#"SELECT *
                FROM calendars
                WHERE principal = ? AND deleted_at IS NOT NULL"#,
            principal
        )
        .fetch_all(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(cals.into_iter().map(Calendar::from).collect())
    }

    async fn _insert_calendar<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        calendar: Calendar,
    ) -> Result<(), Error> {
        let comp_event = calendar.components.contains(&CalendarObjectType::Event);
        let comp_todo = calendar.components.contains(&CalendarObjectType::Todo);
        let comp_journal = calendar.components.contains(&CalendarObjectType::Journal);

        sqlx::query!(
            r#"INSERT INTO calendars (principal, id, displayname, description, "order", color, subscription_url, timezone, timezone_id, push_topic, comp_event, comp_todo, comp_journal)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            calendar.principal,
            calendar.id,
            calendar.displayname,
            calendar.description,
            calendar.order,
            calendar.color,
            calendar.subscription_url,
            calendar.timezone,
            calendar.timezone_id,
            calendar.push_topic,
            comp_event, comp_todo, comp_journal
        )
        .execute(executor)
        .await.map_err(crate::Error::from)?;

        Ok(())
    }

    async fn _update_calendar<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: String,
        id: String,
        calendar: Calendar,
    ) -> Result<(), Error> {
        let comp_event = calendar.components.contains(&CalendarObjectType::Event);
        let comp_todo = calendar.components.contains(&CalendarObjectType::Todo);
        let comp_journal = calendar.components.contains(&CalendarObjectType::Journal);

        let result = sqlx::query!(
            r#"UPDATE calendars SET principal = ?, id = ?, displayname = ?, description = ?, "order" = ?, color = ?, timezone = ?, timezone_id = ?, push_topic = ?, comp_event = ?, comp_todo = ?, comp_journal = ?
                WHERE (principal, id) = (?, ?)"#,
            calendar.principal,
            calendar.id,
            calendar.displayname,
            calendar.description,
            calendar.order,
            calendar.color,
            calendar.timezone,
            calendar.timezone_id,
            calendar.push_topic,
            comp_event, comp_todo, comp_journal,
            principal,
            id
        ).execute(executor).await.map_err(crate::Error::from)?;
        if result.rows_affected() == 0 {
            return Err(rustical_store::Error::NotFound);
        }
        Ok(())
    }

    async fn _delete_calendar<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        match use_trashbin {
            true => sqlx::query!(
                r#"UPDATE calendars SET deleted_at = datetime() WHERE (principal, id) = (?, ?)"#,
                principal,
                id
            )
            .execute(executor)
            .await
            .map_err(crate::Error::from)?,
            false => sqlx::query!(
                r#"DELETE FROM calendars WHERE (principal, id) = (?, ?)"#,
                principal,
                id
            )
            .execute(executor)
            .await
            .map_err(crate::Error::from)?,
        };
        Ok(())
    }

    async fn _restore_calendar<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        id: &str,
    ) -> Result<(), Error> {
        sqlx::query!(
            r"UPDATE calendars SET deleted_at = NULL WHERE (principal, id) = (?, ?)",
            principal,
            id
        )
        .execute(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }

    async fn _get_objects<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        cal_id: &str,
    ) -> Result<Vec<CalendarObject>, Error> {
        sqlx::query_as!(
            CalendarObjectRow,
            "SELECT id, ics FROM calendarobjects WHERE principal = ? AND cal_id = ? AND deleted_at IS NULL",
            principal,
            cal_id
        )
        .fetch_all(executor)
        .await.map_err(crate::Error::from)?
        .into_iter()
        .map(|row| row.try_into())
        .collect()
    }

    async fn _calendar_query<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        cal_id: &str,
        query: CalendarQuery,
    ) -> Result<Vec<CalendarObject>, Error> {
        // We extend our query interval by one day in each direction since we really don't want to
        // miss any objects because of timezone differences
        // I've previously tried NaiveDate::MIN,MAX, but it seems like sqlite cannot handle these
        let start = query.time_start.map(|start| start - TimeDelta::days(1));
        let end = query.time_end.map(|end| end + TimeDelta::days(1));

        sqlx::query_as!(
            CalendarObjectRow,
            r"SELECT id, ics FROM calendarobjects
                WHERE principal = ? AND cal_id = ? AND deleted_at IS NULL
                    AND (last_occurence IS NULL OR ? IS NULL OR last_occurence >= date(?))
                    AND (first_occurence IS NULL OR ? IS NULL OR first_occurence <= date(?))
            ",
            principal,
            cal_id,
            start,
            start,
            end,
            end,
        )
        .fetch_all(executor)
        .await
        .map_err(crate::Error::from)?
        .into_iter()
        .map(|row| row.try_into())
        .collect()
    }

    async fn _get_object<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<CalendarObject, Error> {
        sqlx::query_as!(
            CalendarObjectRow,
            "SELECT id, ics FROM calendarobjects WHERE (principal, cal_id, id) = (?, ?, ?)",
            principal,
            cal_id,
            object_id
        )
        .fetch_one(executor)
        .await
        .map_err(crate::Error::from)?
        .try_into()
    }

    #[instrument]
    async fn _put_object<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: String,
        cal_id: String,
        object: CalendarObject,
        overwrite: bool,
    ) -> Result<(), Error> {
        // TODO: Prevent objects from being commited to a subscription calendar
        let (object_id, ics) = (object.get_id(), object.get_ics());

        let first_occurence = object
            .get_first_occurence()
            .ok()
            .flatten()
            .as_ref()
            .map(CalDateTime::date);
        let last_occurence = object
            .get_last_occurence()
            .ok()
            .flatten()
            .as_ref()
            .map(CalDateTime::date);
        let etag = object.get_etag();
        let object_type = object.get_object_type() as u8;

        (if overwrite {
            sqlx::query!(
                "REPLACE INTO calendarobjects (principal, cal_id, id, ics, first_occurence, last_occurence, etag, object_type) VALUES (?, ?, ?, ?, date(?), date(?), ?, ?)",
                principal,
                cal_id,
                object_id,
                ics,
                first_occurence,
                last_occurence,
                etag,
                object_type,
            )
        } else {
            // If the object already exists a database error is thrown and handled in error.rs
            sqlx::query!(
                "INSERT INTO calendarobjects (principal, cal_id, id, ics, first_occurence, last_occurence, etag, object_type) VALUES (?, ?, ?, ?, date(?), date(?), ?, ?)",
                principal,
                cal_id,
                object_id,
                ics,
                first_occurence,
                last_occurence,
                etag,
                object_type,
            )
        })
        .execute(executor)
        .await
        .map_err(crate::Error::from)?;

        Ok(())
    }

    #[instrument]
    async fn _delete_object<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        cal_id: &str,
        id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        match use_trashbin {
            true => {
                sqlx::query!(
                    "UPDATE calendarobjects SET deleted_at = datetime(), updated_at = datetime() WHERE (principal, cal_id, id) = (?, ?, ?)",
                    principal,
                    cal_id,
                    id
                )
                .execute(executor)
                .await.map_err(crate::Error::from)?;
            }
            false => {
                sqlx::query!(
                    "DELETE FROM calendarobjects WHERE cal_id = ? AND id = ?",
                    cal_id,
                    id
                )
                .execute(executor)
                .await
                .map_err(crate::Error::from)?;
            }
        };
        Ok(())
    }

    #[instrument]
    async fn _restore_object<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<(), Error> {
        sqlx::query!(
            r#"UPDATE calendarobjects SET deleted_at = NULL, updated_at = datetime() WHERE (principal, cal_id, id) = (?, ?, ?)"#,
            principal,
            cal_id,
            object_id
        )
        .execute(executor)
        .await.map_err(crate::Error::from)?;
        Ok(())
    }

    async fn _sync_changes<'a, A: Acquire<'a, Database = Sqlite>>(
        acquire: A,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<CalendarObject>, Vec<String>, i64), Error> {
        struct Row {
            object_id: String,
            synctoken: i64,
        }

        let mut conn = acquire.acquire().await.map_err(crate::Error::from)?;

        let changes = sqlx::query_as!(
            Row,
            r#"
                SELECT DISTINCT object_id, max(0, synctoken) as "synctoken!: i64" from calendarobjectchangelog
                WHERE synctoken > ?
                ORDER BY synctoken ASC
            "#,
            synctoken
        )
        .fetch_all(&mut *conn)
        .await.map_err(crate::Error::from)?;

        let mut objects = vec![];
        let mut deleted_objects = vec![];

        let new_synctoken = changes
            .last()
            .map(|&Row { synctoken, .. }| synctoken)
            .unwrap_or(0);

        for Row { object_id, .. } in changes {
            match Self::_get_object(&mut *conn, principal, cal_id, &object_id).await {
                Ok(object) => objects.push(object),
                Err(rustical_store::Error::NotFound) => deleted_objects.push(object_id),
                Err(err) => return Err(err),
            }
        }

        Ok((objects, deleted_objects, new_synctoken))
    }
}

#[async_trait]
impl CalendarStore for SqliteCalendarStore {
    #[instrument]
    async fn get_calendar(&self, principal: &str, id: &str) -> Result<Calendar, Error> {
        Self::_get_calendar(&self.db, principal, id).await
    }

    #[instrument]
    async fn get_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        Self::_get_calendars(&self.db, principal).await
    }

    #[instrument]
    async fn get_deleted_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        Self::_get_deleted_calendars(&self.db, principal).await
    }

    #[instrument]
    async fn insert_calendar(&self, calendar: Calendar) -> Result<(), Error> {
        Self::_insert_calendar(&self.db, calendar).await
    }

    #[instrument]
    async fn update_calendar(
        &self,
        principal: String,
        id: String,
        calendar: Calendar,
    ) -> Result<(), Error> {
        Self::_update_calendar(&self.db, principal, id, calendar).await
    }

    // Does not actually delete the calendar but just disables it
    #[instrument]
    async fn delete_calendar(
        &self,
        principal: &str,
        id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await.map_err(crate::Error::from)?;

        let cal = match Self::_get_calendar(&mut *tx, principal, id).await {
            Ok(cal) => Some(cal),
            Err(Error::NotFound) => None,
            Err(err) => return Err(err),
        };

        Self::_delete_calendar(&mut *tx, principal, id, use_trashbin).await?;
        tx.commit().await.map_err(crate::Error::from)?;

        if let Some(cal) = cal {
            if let Err(err) = self.sender.try_send(CollectionOperation {
                r#type: CollectionOperationType::Delete,
                domain: rustical_store::CollectionOperationDomain::Calendar,
                topic: cal.push_topic,
                sync_token: None,
            }) {
                error!("Push notification about deleted calendar failed: {err}");
            };
        }
        Ok(())
    }

    #[instrument]
    async fn restore_calendar(&self, principal: &str, id: &str) -> Result<(), Error> {
        Self::_restore_calendar(&self.db, principal, id).await
    }

    #[instrument]
    async fn calendar_query(
        &self,
        principal: &str,
        cal_id: &str,
        query: CalendarQuery,
    ) -> Result<Vec<CalendarObject>, Error> {
        Self::_calendar_query(&self.db, principal, cal_id, query).await
    }

    #[instrument]
    async fn get_objects(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<Vec<CalendarObject>, Error> {
        Self::_get_objects(&self.db, principal, cal_id).await
    }

    #[instrument]
    async fn get_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<CalendarObject, Error> {
        Self::_get_object(&self.db, principal, cal_id, object_id).await
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

        let object_id = object.get_id().to_owned();

        Self::_put_object(
            &mut *tx,
            principal.to_owned(),
            cal_id.to_owned(),
            object,
            overwrite,
        )
        .await?;

        let synctoken = log_object_operation(
            &mut tx,
            &principal,
            &cal_id,
            &object_id,
            ChangeOperation::Add,
        )
        .await?;

        tx.commit().await.map_err(crate::Error::from)?;

        if let Err(err) = self.sender.try_send(CollectionOperation {
            r#type: CollectionOperationType::Object,
            domain: rustical_store::CollectionOperationDomain::Calendar,
            topic: self.get_calendar(&principal, &cal_id).await?.push_topic,
            sync_token: Some(synctoken),
        }) {
            error!("Push notification about deleted calendar failed: {err}");
        };
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

        Self::_delete_object(&mut *tx, principal, cal_id, id, use_trashbin).await?;

        let synctoken =
            log_object_operation(&mut tx, principal, cal_id, id, ChangeOperation::Delete).await?;
        tx.commit().await.map_err(crate::Error::from)?;

        if let Err(err) = self.sender.try_send(CollectionOperation {
            r#type: CollectionOperationType::Object,
            domain: rustical_store::CollectionOperationDomain::Calendar,
            topic: self.get_calendar(principal, cal_id).await?.push_topic,
            sync_token: Some(synctoken),
        }) {
            error!("Push notification about deleted calendar failed: {err}");
        };
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

        Self::_restore_object(&mut *tx, principal, cal_id, object_id).await?;

        let synctoken =
            log_object_operation(&mut tx, principal, cal_id, object_id, ChangeOperation::Add)
                .await?;
        tx.commit().await.map_err(crate::Error::from)?;

        if let Err(err) = self.sender.try_send(CollectionOperation {
            r#type: CollectionOperationType::Object,
            domain: rustical_store::CollectionOperationDomain::Calendar,
            topic: self.get_calendar(principal, cal_id).await?.push_topic,
            sync_token: Some(synctoken),
        }) {
            error!("Push notification about deleted calendar failed: {err}");
        };
        Ok(())
    }

    #[instrument]
    async fn sync_changes(
        &self,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<CalendarObject>, Vec<String>, i64), Error> {
        Self::_sync_changes(&self.db, principal, cal_id, synctoken).await
    }

    fn is_read_only(&self, _cal_id: &str) -> bool {
        false
    }
}

// Logs an operation to the events
async fn log_object_operation(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &str,
    cal_id: &str,
    object_id: &str,
    operation: ChangeOperation,
) -> Result<String, Error> {
    struct Synctoken {
        synctoken: i64,
    }
    let Synctoken { synctoken } = sqlx::query_as!(
        Synctoken,
        r#"
        UPDATE calendars
        SET synctoken = synctoken + 1
        WHERE (principal, id) = (?1, ?2)
        RETURNING synctoken"#,
        principal,
        cal_id
    )
    .fetch_one(&mut **tx)
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
    Ok(format_synctoken(synctoken))
}
