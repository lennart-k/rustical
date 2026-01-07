#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
mod timestamp;
pub use timestamp::*;

mod icalendar;
pub use icalendar::*;

mod error;
pub use error::Error;

mod address_object;
pub use address_object::AddressObject;
