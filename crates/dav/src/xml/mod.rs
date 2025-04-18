pub mod multistatus;
mod propfind;
mod resourcetype;
pub mod tag_list;
use derive_more::derive::From;
pub use multistatus::MultistatusElement;
pub use propfind::{PropElement, PropfindElement, PropfindType, Propname};
pub use resourcetype::{Resourcetype, ResourcetypeInner};
use rustical_xml::{XmlDeserialize, XmlSerialize};
pub use tag_list::TagList;
pub mod sync_collection;

#[derive(XmlDeserialize, XmlSerialize, Debug, Clone, From, PartialEq)]
pub struct HrefElement {
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub href: String,
}

impl HrefElement {
    pub fn new(href: String) -> Self {
        Self { href }
    }
}
