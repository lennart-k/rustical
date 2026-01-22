#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
mod timestamp;
use caldata::parser::ParserError;
pub use timestamp::*;

mod calendar_object;
pub use calendar_object::*;

mod address_object;
pub use address_object::AddressObject;

pub type Error = ParserError;
