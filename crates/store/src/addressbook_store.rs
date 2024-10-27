use crate::{
    error::Error,
    model::{AddressObject, Addressbook},
};
use async_trait::async_trait;

#[async_trait]
pub trait AddressbookStore: Send + Sync + 'static {
    async fn get_addressbook(&self, principal: &str, id: &str) -> Result<Addressbook, Error>;
    async fn get_addressbooks(&self, principal: &str) -> Result<Vec<Addressbook>, Error>;

    async fn update_addressbook(
        &mut self,
        principal: String,
        id: String,
        addressbook: Addressbook,
    ) -> Result<(), Error>;
    async fn insert_addressbook(&mut self, addressbook: Addressbook) -> Result<(), Error>;
    async fn delete_addressbook(
        &mut self,
        principal: &str,
        name: &str,
        use_trashbin: bool,
    ) -> Result<(), Error>;
    async fn restore_addressbook(&mut self, principal: &str, name: &str) -> Result<(), Error>;

    async fn sync_changes(
        &self,
        principal: &str,
        addressbook_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<AddressObject>, Vec<String>, i64), Error>;

    async fn get_objects(
        &self,
        principal: &str,
        addressbook_id: &str,
    ) -> Result<Vec<AddressObject>, Error>;
    async fn get_object(
        &self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
    ) -> Result<AddressObject, Error>;
    async fn put_object(
        &mut self,
        principal: String,
        addressbook_id: String,
        object: AddressObject,
    ) -> Result<(), Error>;
    async fn delete_object(
        &mut self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error>;
    async fn restore_object(
        &mut self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
    ) -> Result<(), Error>;
}
