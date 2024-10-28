use crate::{
    addressbook::{AddressObject, Addressbook},
    Error,
};
use async_trait::async_trait;

#[async_trait]
pub trait AddressbookStore: Send + Sync + 'static {
    async fn get_addressbook(&self, principal: &str, id: &str) -> Result<Addressbook, Error>;
    async fn get_addressbooks(&self, principal: &str) -> Result<Vec<Addressbook>, Error>;

    async fn update_addressbook(
        &self,
        principal: String,
        id: String,
        addressbook: Addressbook,
    ) -> Result<(), Error>;
    async fn insert_addressbook(&self, addressbook: Addressbook) -> Result<(), Error>;
    async fn delete_addressbook(
        &self,
        principal: &str,
        name: &str,
        use_trashbin: bool,
    ) -> Result<(), Error>;
    async fn restore_addressbook(&self, principal: &str, name: &str) -> Result<(), Error>;

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
        &self,
        principal: String,
        addressbook_id: String,
        object: AddressObject,
        overwrite: bool,
    ) -> Result<(), Error>;
    async fn delete_object(
        &self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error>;
    async fn restore_object(
        &self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
    ) -> Result<(), Error>;
}
