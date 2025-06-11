use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Constructor;
use rustical_ical::CalendarObject;

use crate::{
    Calendar, CalendarStore, Error, calendar_store::CalendarQuery,
    contact_birthday_store::BIRTHDAYS_PREFIX,
};

#[derive(Debug, Constructor)]
pub struct CombinedCalendarStore<CS: CalendarStore, BS: CalendarStore> {
    cal_store: Arc<CS>,
    birthday_store: Arc<BS>,
}

impl<CS: CalendarStore, BS: CalendarStore> Clone for CombinedCalendarStore<CS, BS> {
    fn clone(&self) -> Self {
        Self {
            cal_store: self.cal_store.clone(),
            birthday_store: self.birthday_store.clone(),
        }
    }
}

#[async_trait]
impl<CS: CalendarStore, BS: CalendarStore> CalendarStore for CombinedCalendarStore<CS, BS> {
    #[inline]
    async fn get_calendar(&self, principal: &str, id: &str) -> Result<Calendar, Error> {
        if id.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store.get_calendar(principal, id).await
        } else {
            self.cal_store.get_calendar(principal, id).await
        }
    }

    #[inline]
    async fn update_calendar(
        &self,
        principal: String,
        id: String,
        calendar: Calendar,
    ) -> Result<(), crate::Error> {
        if id.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store
                .update_calendar(principal, id, calendar)
                .await
        } else {
            self.cal_store
                .update_calendar(principal, id, calendar)
                .await
        }
    }

    #[inline]
    async fn insert_calendar(&self, calendar: Calendar) -> Result<(), Error> {
        if calendar.id.starts_with(BIRTHDAYS_PREFIX) {
            Err(Error::ReadOnly)
        } else {
            self.cal_store.insert_calendar(calendar).await
        }
    }

    #[inline]
    async fn get_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        Ok([
            self.cal_store.get_calendars(principal).await?,
            self.birthday_store.get_calendars(principal).await?,
        ]
        .concat())
    }

    #[inline]
    async fn delete_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        if cal_id.starts_with(BIRTHDAYS_PREFIX) {
            Err(Error::ReadOnly)
        } else {
            self.birthday_store
                .delete_object(principal, cal_id, object_id, use_trashbin)
                .await
        }
    }

    #[inline]
    async fn get_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<CalendarObject, Error> {
        if cal_id.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store
                .get_object(principal, cal_id, object_id)
                .await
        } else {
            self.cal_store
                .get_object(principal, cal_id, object_id)
                .await
        }
    }

    #[inline]
    async fn sync_changes(
        &self,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<CalendarObject>, Vec<String>, i64), Error> {
        if cal_id.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store
                .sync_changes(principal, cal_id, synctoken)
                .await
        } else {
            self.cal_store
                .sync_changes(principal, cal_id, synctoken)
                .await
        }
    }

    #[inline]
    async fn get_objects(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<Vec<CalendarObject>, Error> {
        if cal_id.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store.get_objects(principal, cal_id).await
        } else {
            self.cal_store.get_objects(principal, cal_id).await
        }
    }

    #[inline]
    async fn calendar_query(
        &self,
        principal: &str,
        cal_id: &str,
        query: CalendarQuery,
    ) -> Result<Vec<CalendarObject>, Error> {
        if cal_id.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store
                .calendar_query(principal, cal_id, query)
                .await
        } else {
            self.cal_store
                .calendar_query(principal, cal_id, query)
                .await
        }
    }

    #[inline]
    async fn restore_calendar(&self, principal: &str, name: &str) -> Result<(), Error> {
        if name.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store.restore_calendar(principal, name).await
        } else {
            self.cal_store.restore_calendar(principal, name).await
        }
    }

    #[inline]
    async fn delete_calendar(
        &self,
        principal: &str,
        name: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        if name.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store
                .delete_calendar(principal, name, use_trashbin)
                .await
        } else {
            self.cal_store
                .delete_calendar(principal, name, use_trashbin)
                .await
        }
    }

    #[inline]
    async fn get_deleted_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        Ok([
            self.birthday_store.get_deleted_calendars(principal).await?,
            self.cal_store.get_deleted_calendars(principal).await?,
        ]
        .concat())
    }

    #[inline]
    async fn restore_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<(), Error> {
        if cal_id.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store
                .restore_object(principal, cal_id, object_id)
                .await
        } else {
            self.cal_store
                .restore_object(principal, cal_id, object_id)
                .await
        }
    }

    #[inline]
    async fn put_object(
        &self,
        principal: String,
        cal_id: String,
        object: CalendarObject,
        overwrite: bool,
    ) -> Result<(), Error> {
        if cal_id.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store
                .put_object(principal, cal_id, object, overwrite)
                .await
        } else {
            self.cal_store
                .put_object(principal, cal_id, object, overwrite)
                .await
        }
    }

    #[inline]
    fn is_read_only(&self, cal_id: &str) -> bool {
        if cal_id.starts_with(BIRTHDAYS_PREFIX) {
            self.birthday_store.is_read_only(cal_id)
        } else {
            self.cal_store.is_read_only(cal_id)
        }
    }
}

