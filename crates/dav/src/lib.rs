#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
pub mod error;
pub mod extensions;
pub mod header;
pub mod namespace;
pub mod privileges;
pub mod resource;
pub mod resources;
pub mod xml;
pub use error::Error;

pub trait Principal: std::fmt::Debug + Clone + Send + Sync + 'static {
    fn get_id(&self) -> &str;
}
