use std::{collections::HashMap, sync::Arc};

use crate::{
    calendar::CalendarObjectType, AddressObject, Addressbook, AddressbookStore, Calendar,
    CalendarObject, CalendarStore, Error,
};
use async_trait::async_trait;
use derive_more::derive::Constructor;
use sha2::{Digest, Sha256};

#[derive(Constructor, Clone)]
pub struct ContactBirthdayStore<AS: AddressbookStore>(Arc<AS>);

fn birthday_calendar(addressbook: Addressbook) -> Calendar {
    Calendar {
        principal: addressbook.principal,
        id: addressbook.id,
        displayname: addressbook
            .displayname
            .map(|name| format!("{} birthdays", name)),
        order: 0,
        description: None,
        color: None,
        timezone: None,
        timezone_id: None,
        deleted_at: addressbook.deleted_at,
        synctoken: addressbook.synctoken,
        subscription_url: None,
        push_topic: {
            let mut hasher = Sha256::new();
            hasher.update("birthdays");
            hasher.update(addressbook.push_topic);
            format!("{:x}", hasher.finalize())
        },
        components: vec![CalendarObjectType::Event],
    }
}

#[async_trait]
impl<AS: AddressbookStore> CalendarStore for ContactBirthdayStore<AS> {
    async fn get_calendar(&self, principal: &str, id: &str) -> Result<Calendar, Error> {
        let addressbook = self.0.get_addressbook(principal, id).await?;
        Ok(birthday_calendar(addressbook))
    }
    async fn get_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        let addressbooks = self.0.get_addressbooks(principal).await?;
        Ok(addressbooks.into_iter().map(birthday_calendar).collect())
    }

    async fn get_deleted_calendars(&self, principal: &str) -> Result<Vec<Calendar>, Error> {
        let addressbooks = self.0.get_deleted_addressbooks(principal).await?;
        Ok(addressbooks.into_iter().map(birthday_calendar).collect())
    }

    async fn update_calendar(
        &self,
        _principal: String,
        _id: String,
        _calendar: Calendar,
    ) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    async fn insert_calendar(&self, _calendar: Calendar) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }
    async fn delete_calendar(
        &self,
        _principal: &str,
        _name: &str,
        _use_trashbin: bool,
    ) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    async fn restore_calendar(&self, _principal: &str, _name: &str) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    async fn sync_changes(
        &self,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<CalendarObject>, Vec<String>, i64), Error> {
        let (objects, deleted_objects, new_synctoken) =
            self.0.sync_changes(principal, cal_id, synctoken).await?;
        let objects: Result<Vec<Option<CalendarObject>>, Error> = objects
            .iter()
            .map(AddressObject::get_birthday_object)
            .collect();
        let objects = objects?.into_iter().flatten().collect();

        Ok((objects, deleted_objects, new_synctoken))
    }

    async fn get_objects(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<Vec<CalendarObject>, Error> {
        let objects: Result<Vec<HashMap<&'static str, CalendarObject>>, Error> = self
            .0
            .get_objects(principal, cal_id)
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

    async fn get_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<CalendarObject, Error> {
        let (addressobject_id, date_type) = object_id.rsplit_once("-").ok_or(Error::NotFound)?;
        dbg!(addressobject_id, date_type);
        self.0
            .get_object(principal, cal_id, addressobject_id)
            .await?
            .get_significant_dates()?
            .remove(date_type)
            .ok_or(Error::NotFound)
    }

    async fn put_object(
        &self,
        _principal: String,
        _cal_id: String,
        _object: CalendarObject,
        _overwrite: bool,
    ) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    async fn delete_object(
        &self,
        _principal: &str,
        _cal_id: &str,
        _object_id: &str,
        _use_trashbin: bool,
    ) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    async fn restore_object(
        &self,
        _principal: &str,
        _cal_id: &str,
        _object_id: &str,
    ) -> Result<(), Error> {
        Err(Error::ReadOnly)
    }

    fn is_read_only(&self) -> bool {
        true
    }
}
