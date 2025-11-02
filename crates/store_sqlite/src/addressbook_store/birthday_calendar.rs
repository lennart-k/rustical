use crate::addressbook_store::SqliteAddressbookStore;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use rustical_ical::{AddressObject, CalendarObject, CalendarObjectType};
use rustical_store::{
    Addressbook, AddressbookStore, Calendar, CalendarMetadata, CalendarStore, CollectionMetadata,
    Error, PrefixedCalendarStore,
};
use sha2::{Digest, Sha256};
use sqlx::{Executor, Sqlite};
use std::collections::HashMap;
use tracing::instrument;

pub const BIRTHDAYS_PREFIX: &str = "_birthdays_";

struct BirthdayCalendarJoinRow {
    principal: String,
    id: String,
    displayname: Option<String>,
    description: Option<String>,
    order: i64,
    color: Option<String>,
    timezone_id: Option<String>,
    deleted_at: Option<NaiveDateTime>,
    push_topic: String,

    addr_synctoken: i64,
}

impl From<BirthdayCalendarJoinRow> for Calendar {
    fn from(value: BirthdayCalendarJoinRow) -> Self {
        Self {
            principal: value.principal,
            id: format!("{}{}", BIRTHDAYS_PREFIX, value.id),
            meta: CalendarMetadata {
                displayname: value.displayname,
                order: value.order,
                description: value.description,
                color: value.color,
            },
            deleted_at: value.deleted_at,
            components: vec![CalendarObjectType::Event],
            timezone_id: value.timezone_id,
            synctoken: value.addr_synctoken,
            subscription_url: None,
            push_topic: value.push_topic,
        }
    }
}

impl PrefixedCalendarStore for SqliteAddressbookStore {
    const PREFIX: &'static str = BIRTHDAYS_PREFIX;
}

impl SqliteAddressbookStore {
    #[instrument]
    pub async fn _get_birthday_calendar<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        id: &str,
        show_deleted: bool,
    ) -> Result<Calendar, Error> {
        let cal = sqlx::query_as!(
            BirthdayCalendarJoinRow,
            r#"SELECT principal, id, displayname, description, "order", color, timezone_id, deleted_at, addr_synctoken, push_topic
                FROM birthday_calendars
                INNER JOIN (
                    SELECT principal AS addr_principal,
                        id AS addr_id,
                        synctoken AS addr_synctoken
                    FROM addressbooks
                    ) ON (principal, id) = (addr_principal, addr_id)
                WHERE (principal, id) = (?, ?)
                AND ((deleted_at IS NULL) OR ?)
            "#,
            principal,
            id,
            show_deleted
        )
        .fetch_one(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(cal.into())
    }

    #[instrument]
    pub async fn _get_birthday_calendars<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        deleted: bool,
    ) -> Result<Vec<Calendar>, Error> {
        Ok(
        sqlx::query_as!(
            BirthdayCalendarJoinRow,
            r#"SELECT principal, id, displayname, description, "order", color, timezone_id, deleted_at, addr_synctoken, push_topic
                FROM birthday_calendars
                INNER JOIN (
                    SELECT principal AS addr_principal,
                        id AS addr_id,
                        synctoken AS addr_synctoken
                    FROM addressbooks
                    ) ON (principal, id) = (addr_principal, addr_id)
                WHERE principal = ?
                AND (
                    (deleted_at IS NULL AND NOT ?) -- not deleted, want not deleted
                    OR (deleted_at IS NOT NULL AND ?) -- deleted, want deleted
                )
            "#,
            principal,
            deleted,
            deleted
        )
        .fetch_all(executor)
        .await
        .map_err(crate::Error::from).map(|cals| cals.into_iter().map(BirthdayCalendarJoinRow::into).collect())?)
    }

    #[instrument]
    pub async fn _insert_birthday_calendar<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        addressbook: Addressbook,
    ) -> Result<(), rustical_store::Error> {
        let birthday_name = addressbook
            .displayname
            .map(|name| format!("{name} birthdays"));
        let birthday_push_topic = {
            let mut hasher = Sha256::new();
            hasher.update("birthdays");
            hasher.update(addressbook.push_topic);
            format!("{:x}", hasher.finalize())
        };

        sqlx::query!(
            r#"INSERT INTO birthday_calendars (principal, id, displayname, push_topic)
                VALUES (?, ?, ?, ?)"#,
            addressbook.principal,
            addressbook.id,
            birthday_name,
            birthday_push_topic,
        )
        .execute(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn _update_birthday_calendar<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        calendar: &Calendar,
    ) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE birthday_calendars SET principal = ?, id = ?, displayname = ?, description = ?, "order" = ?, color = ?, timezone_id = ?, push_topic = ?
                WHERE (principal, id) = (?, ?)"#,
            calendar.principal,
            calendar.id,
            calendar.meta.displayname,
            calendar.meta.description,
            calendar.meta.order,
            calendar.meta.color,
            calendar.timezone_id,
            calendar.push_topic,
            principal,
            calendar.id,
        ).execute(executor).await.map_err(crate::Error::from)?;
        if result.rows_affected() == 0 {
            return Err(rustical_store::Error::NotFound);
        }
        Ok(())
    }
}

#[async_trait]
impl CalendarStore for SqliteAddressbookStore {
    #[instrument]
    async fn get_calendar(
        &self,
        principal: &str,
        id: &str,
        show_deleted: bool,
    ) -> Result<Calendar, Error> {
        let id = id.strip_prefix(BIRTHDAYS_PREFIX).ok_or(Error::NotFound)?;
        Self::_get_birthday_calendar(&self.db, principal, id, show_deleted).await
    }

    #[instrument]
    async fn get_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        Self::_get_birthday_calendars(&self.db, principal, false).await
    }

    #[instrument]
    async fn get_deleted_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        Self::_get_birthday_calendars(&self.db, principal, true).await
    }

    #[instrument]
    async fn update_calendar(
        &self,
        principal: String,
        id: String,
        mut calendar: Calendar,
    ) -> Result<(), Error> {
        assert_eq!(id, calendar.id);
        calendar.id = calendar
            .id
            .strip_prefix(BIRTHDAYS_PREFIX)
            .ok_or(Error::NotFound)?
            .to_string();
        Self::_update_birthday_calendar(&self.db, &principal, &calendar).await
    }

    #[instrument]
    async fn insert_calendar(&self, _calendar: Calendar) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    #[instrument]
    async fn delete_calendar(
        &self,
        _principal: &str,
        _name: &str,
        _use_trashbin: bool,
    ) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    #[instrument]
    async fn restore_calendar(&self, _principal: &str, _name: &str) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    #[instrument]
    async fn import_calendar(
        &self,
        _calendar: Calendar,
        _objects: Vec<CalendarObject>,
        _merge_existing: bool,
    ) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    #[instrument]
    async fn sync_changes(
        &self,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<CalendarObject>, Vec<String>, i64), Error> {
        let cal_id = cal_id
            .strip_prefix(BIRTHDAYS_PREFIX)
            .ok_or(Error::NotFound)?;
        let (objects, deleted_objects, new_synctoken) =
            AddressbookStore::sync_changes(self, principal, cal_id, synctoken).await?;
        let objects: Result<Vec<Option<CalendarObject>>, rustical_ical::Error> = objects
            .iter()
            .map(AddressObject::get_birthday_object)
            .collect();
        let objects = objects?.into_iter().flatten().collect();

        Ok((objects, deleted_objects, new_synctoken))
    }

    #[instrument]
    async fn calendar_metadata(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<CollectionMetadata, Error> {
        let cal_id = cal_id
            .strip_prefix(BIRTHDAYS_PREFIX)
            .ok_or(Error::NotFound)?;
        self.addressbook_metadata(principal, cal_id).await
    }

    #[instrument]
    async fn get_objects(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<Vec<CalendarObject>, Error> {
        let cal_id = cal_id
            .strip_prefix(BIRTHDAYS_PREFIX)
            .ok_or(Error::NotFound)?;
        let objects: Result<Vec<HashMap<&'static str, CalendarObject>>, rustical_ical::Error> =
            AddressbookStore::get_objects(self, principal, cal_id)
                .await?
                .iter()
                .map(AddressObject::get_significant_dates)
                .collect();
        let objects = objects?
            .into_iter()
            .flat_map(HashMap::into_values)
            .collect();

        Ok(objects)
    }

    #[instrument]
    async fn get_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        show_deleted: bool,
    ) -> Result<CalendarObject, Error> {
        let cal_id = cal_id
            .strip_prefix(BIRTHDAYS_PREFIX)
            .ok_or(Error::NotFound)?;
        let (addressobject_id, date_type) = object_id.rsplit_once('-').ok_or(Error::NotFound)?;
        AddressbookStore::get_object(self, principal, cal_id, addressobject_id, show_deleted)
            .await?
            .get_significant_dates()?
            .remove(date_type)
            .ok_or(Error::NotFound)
    }

    #[instrument]
    async fn put_object(
        &self,
        _principal: String,
        _cal_id: String,
        _object: CalendarObject,
        _overwrite: bool,
    ) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    #[instrument]
    async fn delete_object(
        &self,
        _principal: &str,
        _cal_id: &str,
        _object_id: &str,
        _use_trashbin: bool,
    ) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    #[instrument]
    async fn restore_object(
        &self,
        _principal: &str,
        _cal_id: &str,
        _object_id: &str,
    ) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    fn is_read_only(&self, _cal_id: &str) -> bool {
        true
    }
}
