use crate::{CollectionMetadata, Error, addressbook::Addressbook};
use async_trait::async_trait;
use rustical_ical::AddressObject;

#[async_trait]
pub trait AddressbookStore: Send + Sync + 'static {
    async fn get_addressbook(
        &self,
        principal: &str,
        id: &str,
        show_deleted: bool,
    ) -> Result<Addressbook, Error>;
    async fn get_addressbooks(&self, principal: &str) -> Result<Vec<Addressbook>, Error>;
    async fn get_deleted_addressbooks(&self, principal: &str) -> Result<Vec<Addressbook>, Error>;

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

    async fn addressbook_metadata(
        &self,
        principal: &str,
        addressbook_id: &str,
    ) -> Result<CollectionMetadata, Error>;

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
        show_deleted: bool,
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

    async fn import_addressbook(
        &self,
        addressbook: Addressbook,
        objects: Vec<AddressObject>,
        merge_existing: bool,
    ) -> Result<(), Error>;
}
