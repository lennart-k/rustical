pub mod addressbook;
pub mod addressbook_store;
pub mod calendar_store;
pub mod error;
pub use error::Error;
pub mod auth;
mod calendar;
mod combined_calendar_store;
mod contact_birthday_store;
mod secret;
mod subscription_store;
pub mod synctoken;

#[cfg(test)]
pub mod tests;

pub use addressbook_store::AddressbookStore;
pub use calendar_store::CalendarStore;
pub use combined_calendar_store::CombinedCalendarStore;
pub use contact_birthday_store::ContactBirthdayStore;
pub use secret::Secret;
pub use subscription_store::*;

pub use addressbook::Addressbook;
pub use calendar::{Calendar, CalendarMetadata};

#[derive(Debug, Clone)]
pub enum CollectionOperationInfo {
    // Sync-Token increased
    Content { sync_token: String },
    // Collection deleted
    Delete,
}

#[derive(Debug, Clone)]
pub struct CollectionOperation {
    pub topic: String,
    pub data: CollectionOperationInfo,
}

#[derive(Default, Debug, Clone)]
pub struct CollectionMetadata {
    pub len: usize,
    pub deleted_len: usize,
    pub size: u64,
    pub deleted_size: u64,
}
