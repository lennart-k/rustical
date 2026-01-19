use super::ChangeOperation;
use crate::BEGIN_IMMEDIATE;
use async_trait::async_trait;
use chrono::TimeDelta;
use derive_more::derive::Constructor;
use ical::parser::ParserError;
use ical::types::CalDateTime;
use regex::Regex;
use rustical_ical::{CalendarObject, CalendarObjectType};
use rustical_store::calendar_store::CalendarQuery;
use rustical_store::synctoken::format_synctoken;
use rustical_store::{Calendar, CalendarMetadata, CalendarStore, CollectionMetadata, Error};
use rustical_store::{CollectionOperation, CollectionOperationInfo};
use sqlx::types::chrono::NaiveDateTime;
use sqlx::{Acquire, Executor, Sqlite, SqlitePool, Transaction};
use tokio::sync::mpsc::Sender;
use tracing::{error, error_span, instrument, warn};

#[derive(Debug, Clone)]
struct CalendarObjectRow {
    id: String,
    ics: String,
    uid: String,
}

impl From<CalendarObjectRow> for (String, Result<CalendarObject, ParserError>) {
    fn from(row: CalendarObjectRow) -> Self {
        let result = CalendarObject::from_ics(row.ics).inspect(|object| {
            if object.get_uid() != row.uid {
                warn!(
                    "Calendar object {}.ics: UID={} and row uid={} do not match",
                    row.id,
                    object.get_uid(),
                    row.uid
                );
            }
        });

        (row.id, result)
    }
}

impl TryFrom<CalendarObjectRow> for (String, CalendarObject) {
    type Error = rustical_store::Error;

    fn try_from(row: CalendarObjectRow) -> Result<Self, Self::Error> {
        let object = CalendarObject::from_ics(row.ics)?;
        if object.get_uid() != row.uid {
            warn!(
                "Calendar object {}.ics: UID={} and row uid={} do not match",
                row.id,
                object.get_uid(),
                row.uid
            );
        }
        Ok((row.id, object))
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
            meta: CalendarMetadata {
                displayname: value.displayname,
                order: value.order,
                description: value.description,
                color: value.color,
            },
            timezone_id: value.timezone_id,
            deleted_at: value.deleted_at,
            synctoken: value.synctoken,
            subscription_url: value.subscription_url,
            push_topic: value.push_topic,
            components,
        }
    }
}

#[derive(Debug, Clone, Constructor)]
pub struct SqliteCalendarStore {
    db: SqlitePool,
    sender: Sender<CollectionOperation>,
    skip_broken: bool,
}

impl SqliteCalendarStore {
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

    fn send_push_notification(&self, data: CollectionOperationInfo, topic: String) {
        if let Err(err) = self.sender.try_send(CollectionOperation { topic, data }) {
            error_span!(
                "Error trying to send calendar update notification:",
                err = format!("{err}"),
            );
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub async fn validate_objects(&self, principal: &str) -> Result<(), Error> {
        let mut success = true;
        for calendar in self.get_calendars(principal).await? {
            for (object_id, res) in Self::_get_objects(&self.db, principal, &calendar.id).await? {
                if let Err(err) = res {
                    warn!(
                        "Invalid calendar object found at {principal}/{cal_id}/{object_id}.ics. Error: {err}",
                        cal_id = calendar.id
                    );
                    success = false;
                }
            }
        }
        if !success {
            if self.skip_broken {
                error!(
                    "Not all calendar objects are valid. Since data_store.sqlite.skip_broken=true they will be hidden. You are still advised to manually remove or repair the object. If you need help feel free to open up an issue on GitHub."
                );
            } else {
                error!(
                    "Not all calendar objects are valid. Since data_store.sqlite.skip_broken=false this causes a panic. Remove or repair the broken objects manually or set data_store.sqlite.skip_broken=false as a temporary solution to ignore the error. If you need help feel free to open up an issue on GitHub."
                );
                panic!();
            }
        }
        Ok(())
    }

    /// In the past exports generated objects with invalid VERSION:4.0
    /// This repair sets them to VERSION:2.0
    #[allow(clippy::missing_panics_doc)]
    pub async fn repair_invalid_version_4_0(&self) -> Result<(), Error> {
        struct Row {
            principal: String,
            cal_id: String,
            id: String,
            ics: String,
        }

        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;

        #[allow(clippy::missing_panics_doc)]
        let version_pattern = Regex::new(r"(?mi)^VERSION:4.0").unwrap();

        let repairs: Vec<Row> = sqlx::query_as!(
            Row,
            r#"SELECT principal, cal_id, id, ics FROM calendarobjects WHERE ics LIKE '%VERSION:4.0%';"#
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::Error::from)?
            .into_iter()
            .filter_map(|mut row| {
                version_pattern.find(&row.ics)?;
                let new_ics = version_pattern.replace(&row.ics, "VERSION:2.0");
                // Safeguard that we really only changed the version
                assert_eq!(row.ics.len(), new_ics.len());
                row.ics = new_ics.to_string();
                Some(row)
            })
            .collect();

        if repairs.is_empty() {
            return Ok(());
        }
        warn!(
            "Found {} calendar objects with invalid VERSION:4.0. Repairing by setting to VERSION:2.0",
            repairs.len()
        );

        for repair in &repairs {
            // calendarobjectchangelog is used by sync-collection to fetch changes
            // By deleting entries we will later regenerate new entries such that clients will notice
            // the objects have changed
            warn!(
                "Repairing VERSION for {}/{}/{}.ics",
                repair.principal, repair.cal_id, repair.id
            );
            sqlx::query!(
                "DELETE FROM calendarobjectchangelog WHERE (principal, cal_id, object_id) = (?, ?, ?);",
                repair.principal, repair.cal_id, repair.id
            ).execute(&mut *tx).await
            .map_err(crate::Error::from)?;

            sqlx::query!(
                "UPDATE calendarobjects SET ics = ? WHERE (principal, cal_id, id) = (?, ?, ?);",
                repair.ics,
                repair.principal,
                repair.cal_id,
                repair.id
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }

        tx.commit().await.map_err(crate::Error::from)?;

        Ok(())
    }

    // Commit "orphaned" objects to the changelog table
    pub async fn repair_orphans(&self) -> Result<(), Error> {
        struct Row {
            principal: String,
            cal_id: String,
            id: String,
            deleted: bool,
        }

        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;

        let rows = sqlx::query_as!(
            Row,
            r#"
                SELECT principal, cal_id, id, (deleted_at IS NOT NULL) AS "deleted: bool"
                    FROM calendarobjects
                    WHERE (principal, cal_id, id) NOT IN (
                        SELECT DISTINCT principal, cal_id, object_id FROM calendarobjectchangelog
                    )
                ;
                "#,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::Error::from)?;

        for row in rows {
            let operation = if row.deleted {
                ChangeOperation::Delete
            } else {
                ChangeOperation::Add
            };
            warn!(
                "Commiting orphaned calendar object ({},{},{}), deleted={}",
                &row.principal, &row.cal_id, &row.id, &row.deleted
            );
            Self::log_object_operation(&mut tx, &row.principal, &row.cal_id, &row.id, operation)
                .await?;
        }
        tx.commit().await.map_err(crate::Error::from)?;

        Ok(())
    }

    async fn _get_calendar<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        id: &str,
        show_deleted: bool,
    ) -> Result<Calendar, Error> {
        let cal = sqlx::query_as!(
            CalendarRow,
            r#"SELECT *
                FROM calendars
                WHERE (principal, id) = (?, ?)
                AND ((deleted_at IS NULL) OR ?) "#,
            principal,
            id,
            show_deleted
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
            r#"SELECT principal, id, displayname, "order", description, color, timezone_id, deleted_at, synctoken, subscription_url, push_topic, comp_event, comp_todo, comp_journal
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
            r#"INSERT INTO calendars (principal, id, displayname, description, "order", color, subscription_url, timezone_id, push_topic, comp_event, comp_todo, comp_journal)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            calendar.principal,
            calendar.id,
            calendar.meta.displayname,
            calendar.meta.description,
            calendar.meta.order,
            calendar.meta.color,
            calendar.subscription_url,
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
        principal: &str,
        id: &str,
        calendar: Calendar,
    ) -> Result<(), Error> {
        let comp_event = calendar.components.contains(&CalendarObjectType::Event);
        let comp_todo = calendar.components.contains(&CalendarObjectType::Todo);
        let comp_journal = calendar.components.contains(&CalendarObjectType::Journal);

        let result = sqlx::query!(
            r#"UPDATE calendars SET principal = ?, id = ?, displayname = ?, description = ?, "order" = ?, color = ?, timezone_id = ?, push_topic = ?, comp_event = ?, comp_todo = ?, comp_journal = ?
                WHERE (principal, id) = (?, ?)"#,
            calendar.principal,
            calendar.id,
            calendar.meta.displayname,
            calendar.meta.description,
            calendar.meta.order,
            calendar.meta.color,
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
        if use_trashbin {
            sqlx::query!(
                r#"UPDATE calendars SET deleted_at = datetime() WHERE (principal, id) = (?, ?)"#,
                principal,
                id
            )
            .execute(executor)
            .await
            .map_err(crate::Error::from)?
        } else {
            sqlx::query!(
                r#"DELETE FROM calendars WHERE (principal, id) = (?, ?)"#,
                principal,
                id
            )
            .execute(executor)
            .await
            .map_err(crate::Error::from)?
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

    async fn _list_objects<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        cal_id: &str,
    ) -> Result<Vec<(u64, bool)>, rustical_store::Error> {
        struct ObjectEntry {
            length: u64,
            deleted: bool,
        }
        Ok(sqlx::query_as!(
            ObjectEntry,
            "SELECT length(ics) AS 'length!: u64', deleted_at AS 'deleted!: bool' FROM calendarobjects WHERE principal = ? AND cal_id = ?",
            principal,
            cal_id
        )
        .fetch_all(executor)
        .await.map_err(crate::Error::from)?
        .into_iter()
        .map(|row| (row.length, row.deleted))
        .collect())
    }

    async fn _get_objects<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        cal_id: &str,
    ) -> Result<impl Iterator<Item = (String, Result<CalendarObject, ParserError>)>, Error> {
        Ok(sqlx::query_as!(
            CalendarObjectRow,
            "SELECT id, uid, ics FROM calendarobjects WHERE principal = ? AND cal_id = ? AND deleted_at IS NULL",
            principal,
            cal_id
        )
        .fetch_all(executor)
        .await.map_err(crate::Error::from)?
        .into_iter()
        .map(Into::into)
        )
    }

    async fn _calendar_query<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        cal_id: &str,
        query: CalendarQuery,
    ) -> Result<impl Iterator<Item = (String, Result<CalendarObject, ParserError>)>, Error> {
        // We extend our query interval by one day in each direction since we really don't want to
        // miss any objects because of timezone differences
        // I've previously tried NaiveDate::MIN,MAX, but it seems like sqlite cannot handle these
        let start = query.time_start.map(|start| start - TimeDelta::days(1));
        let end = query.time_end.map(|end| end + TimeDelta::days(1));

        Ok(sqlx::query_as!(
            CalendarObjectRow,
            r"SELECT id, uid, ics FROM calendarobjects
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
        .map(Into::into))
    }

    async fn _get_object<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        show_deleted: bool,
    ) -> Result<CalendarObject, Error> {
        let (row_id, object) = sqlx::query_as!(
            CalendarObjectRow,
            "SELECT id, uid, ics FROM calendarobjects WHERE (principal, cal_id, id) = (?, ?, ?) AND ((deleted_at IS NULL) OR ?)",
            principal,
            cal_id,
            object_id,
            show_deleted
        )
        .fetch_one(executor)
        .await
        .map_err(crate::Error::from)?
        .try_into()?;
        assert_eq!(object_id, row_id);
        Ok(object)
    }

    #[instrument]
    async fn _put_object<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        object: &CalendarObject,
        overwrite: bool,
    ) -> Result<(), Error> {
        let (uid, ics) = (object.get_uid(), object.get_ics());

        let first_occurence = object
            .get_inner()
            .get_inner()
            .get_first_occurence()
            .as_ref()
            .map(CalDateTime::date_floor);
        let last_occurence = object
            .get_inner()
            .get_inner()
            .get_last_occurence()
            .as_ref()
            .map(CalDateTime::date_ceil);
        let etag = object.get_etag();
        let object_type = object.get_object_type() as u8;

        (if overwrite {
            sqlx::query!(
                "REPLACE INTO calendarobjects (principal, cal_id, id, uid, ics, first_occurence, last_occurence, etag, object_type) VALUES (?, ?, ?, ?, ?, date(?), date(?), ?, ?)",
                principal,
                cal_id,
                object_id,
                uid,
                ics,
                first_occurence,
                last_occurence,
                etag,
                object_type,
            )
        } else {
            // If the object already exists a database error is thrown and handled in error.rs
            sqlx::query!(
                "INSERT INTO calendarobjects (principal, cal_id, id, uid, ics, first_occurence, last_occurence, etag, object_type) VALUES (?, ?, ?, ?, ?, date(?), date(?), ?, ?)",
                principal,
                cal_id,
                object_id,
                uid,
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
        object_id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        if use_trashbin {
            sqlx::query!(
                "UPDATE calendarobjects SET deleted_at = datetime(), updated_at = datetime() WHERE (principal, cal_id, id) = (?, ?, ?)",
                principal,
                cal_id,
                object_id
            )
            .execute(executor)
            .await.map_err(crate::Error::from)?;
        } else {
            sqlx::query!(
                "DELETE FROM calendarobjects WHERE cal_id = ? AND id = ?",
                cal_id,
                object_id
            )
            .execute(executor)
            .await
            .map_err(crate::Error::from)?;
        }
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
        skip_broken: bool,
    ) -> Result<(Vec<(String, CalendarObject)>, Vec<String>, i64), Error> {
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

        let new_synctoken = changes.last().map_or(0, |&Row { synctoken, .. }| synctoken);

        for Row { object_id, .. } in changes {
            match Self::_get_object(&mut *conn, principal, cal_id, &object_id, false).await {
                Ok(object) => objects.push((object_id, object)),
                Err(rustical_store::Error::NotFound) => deleted_objects.push(object_id),
                // Skip broken object
                Err(rustical_store::Error::IcalError(_)) if skip_broken => (),
                Err(err) => return Err(err),
            }
        }

        Ok((objects, deleted_objects, new_synctoken))
    }
}

#[async_trait]
impl CalendarStore for SqliteCalendarStore {
    #[instrument]
    async fn get_calendar(
        &self,
        principal: &str,
        id: &str,
        show_deleted: bool,
    ) -> Result<Calendar, Error> {
        Self::_get_calendar(&self.db, principal, id, show_deleted).await
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
        principal: &str,
        id: &str,
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
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;

        let cal = match Self::_get_calendar(&mut *tx, principal, id, true).await {
            Ok(cal) => Some(cal),
            Err(Error::NotFound) => None,
            Err(err) => return Err(err),
        };

        Self::_delete_calendar(&mut *tx, principal, id, use_trashbin).await?;
        tx.commit().await.map_err(crate::Error::from)?;

        if let Some(cal) = cal {
            self.send_push_notification(CollectionOperationInfo::Delete, cal.push_topic);
        }
        Ok(())
    }

    #[instrument]
    async fn restore_calendar(&self, principal: &str, id: &str) -> Result<(), Error> {
        Self::_restore_calendar(&self.db, principal, id).await
    }

    #[instrument]
    async fn import_calendar(
        &self,
        calendar: Calendar,
        objects: Vec<CalendarObject>,
        merge_existing: bool,
    ) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;

        let existing_cal =
            match Self::_get_calendar(&mut *tx, &calendar.principal, &calendar.id, true).await {
                Ok(cal) => Some(cal),
                Err(Error::NotFound) => None,
                Err(err) => return Err(err),
            };
        if existing_cal.is_some() && !merge_existing {
            return Err(Error::AlreadyExists);
        }
        if existing_cal.is_none() {
            Self::_insert_calendar(&mut *tx, calendar.clone()).await?;
        }

        let mut sync_token = None;
        for object in objects {
            let object_id = object.get_uid();
            Self::_put_object(
                &mut *tx,
                &calendar.principal,
                &calendar.id,
                object_id,
                &object,
                false,
            )
            .await?;

            sync_token = Some(
                Self::log_object_operation(
                    &mut tx,
                    &calendar.principal,
                    &calendar.id,
                    object_id,
                    ChangeOperation::Add,
                )
                .await?,
            );
        }

        tx.commit().await.map_err(crate::Error::from)?;

        if let Some(sync_token) = sync_token {
            self.send_push_notification(
                CollectionOperationInfo::Content { sync_token },
                self.get_calendar(&calendar.principal, &calendar.id, true)
                    .await?
                    .push_topic,
            );
        }
        Ok(())
    }

    #[instrument]
    async fn calendar_query(
        &self,
        principal: &str,
        cal_id: &str,
        query: CalendarQuery,
    ) -> Result<Vec<(String, CalendarObject)>, Error> {
        let objects = Self::_calendar_query(&self.db, principal, cal_id, query).await?;
        if self.skip_broken {
            Ok(objects
                .filter_map(|(id, res)| Some((id, res.ok()?)))
                .collect())
        } else {
            Ok(objects
                .map(|(id, res)| res.map(|obj| (id, obj)))
                .collect::<Result<Vec<_>, _>>()?)
        }
    }

    async fn calendar_metadata(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<CollectionMetadata, Error> {
        let mut sizes = vec![];
        let mut deleted_sizes = vec![];
        for (size, deleted) in Self::_list_objects(&self.db, principal, cal_id).await? {
            if deleted {
                deleted_sizes.push(size);
            } else {
                sizes.push(size);
            }
        }
        Ok(CollectionMetadata {
            len: sizes.len(),
            deleted_len: deleted_sizes.len(),
            size: sizes.iter().sum(),
            deleted_size: deleted_sizes.iter().sum(),
        })
    }

    #[instrument]
    async fn get_objects(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<Vec<(String, CalendarObject)>, Error> {
        let objects = Self::_get_objects(&self.db, principal, cal_id).await?;
        if self.skip_broken {
            Ok(objects
                .filter_map(|(id, res)| Some((id, res.ok()?)))
                .collect())
        } else {
            Ok(objects
                .map(|(id, res)| res.map(|obj| (id, obj)))
                .collect::<Result<Vec<_>, _>>()?)
        }
    }

    #[instrument]
    async fn get_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        show_deleted: bool,
    ) -> Result<CalendarObject, Error> {
        Self::_get_object(&self.db, principal, cal_id, object_id, show_deleted).await
    }

    #[instrument]
    async fn put_objects(
        &self,
        principal: &str,
        cal_id: &str,
        objects: Vec<(String, CalendarObject)>,
        overwrite: bool,
    ) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;

        let calendar = Self::_get_calendar(&mut *tx, principal, cal_id, true).await?;
        if calendar.subscription_url.is_some() {
            // We cannot commit an object to a subscription calendar
            return Err(Error::ReadOnly);
        }

        let mut sync_token = None;
        for (object_id, object) in objects {
            sync_token = Some(
                Self::log_object_operation(
                    &mut tx,
                    principal,
                    cal_id,
                    &object_id,
                    ChangeOperation::Add,
                )
                .await?,
            );
            Self::_put_object(&mut *tx, principal, cal_id, &object_id, &object, overwrite).await?;
        }

        tx.commit().await.map_err(crate::Error::from)?;

        if let Some(sync_token) = sync_token {
            self.send_push_notification(
                CollectionOperationInfo::Content { sync_token },
                self.get_calendar(principal, cal_id, true).await?.push_topic,
            );
        }
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
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;

        Self::_delete_object(&mut *tx, principal, cal_id, id, use_trashbin).await?;

        let sync_token =
            Self::log_object_operation(&mut tx, principal, cal_id, id, ChangeOperation::Delete)
                .await?;
        tx.commit().await.map_err(crate::Error::from)?;

        self.send_push_notification(
            CollectionOperationInfo::Content { sync_token },
            self.get_calendar(principal, cal_id, true).await?.push_topic,
        );

        Ok(())
    }

    #[instrument]
    async fn restore_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;

        Self::_restore_object(&mut *tx, principal, cal_id, object_id).await?;

        let sync_token =
            Self::log_object_operation(&mut tx, principal, cal_id, object_id, ChangeOperation::Add)
                .await?;
        tx.commit().await.map_err(crate::Error::from)?;

        self.send_push_notification(
            CollectionOperationInfo::Content { sync_token },
            self.get_calendar(principal, cal_id, true).await?.push_topic,
        );
        Ok(())
    }

    #[instrument]
    async fn sync_changes(
        &self,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<(String, CalendarObject)>, Vec<String>, i64), Error> {
        Self::_sync_changes(&self.db, principal, cal_id, synctoken, self.skip_broken).await
    }

    fn is_read_only(&self, _cal_id: &str) -> bool {
        false
    }
}
