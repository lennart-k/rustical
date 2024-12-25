pub mod multistatus;
mod propfind;
mod resourcetype;
pub mod tag_list;
use derive_more::derive::From;
pub use multistatus::MultistatusElement;
pub use propfind::{PropElement, PropfindElement, PropfindType, Propname};
pub use resourcetype::Resourcetype;
use rustical_xml::XmlDeserialize;
use serde::Serialize;
pub use tag_list::TagList;

#[derive(XmlDeserialize, Debug, Clone, Serialize, From, PartialEq)]
pub struct HrefElement {
    pub href: String,
}

impl HrefElement {
    pub fn new(href: String) -> Self {
        Self { href }
    }
}
