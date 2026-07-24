#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
pub mod addressbook;
pub mod addressbook_store;
pub mod calendar_store;
pub mod error;
pub use error::Error;
pub mod auth;
mod calendar;
mod combined_calendar_store;
mod secret;
pub mod synctoken;

#[cfg(test)]
pub mod tests;

pub use addressbook_store::*;
pub use calendar_store::*;
pub use combined_calendar_store::{CombinedCalendarStore, PrefixedCalendarStore};
pub use secret::Secret;

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
