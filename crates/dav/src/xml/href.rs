use derive_more::From;
use rustical_xml::{XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, Debug, Clone, From, PartialEq, Eq)]
pub struct HrefElement {
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub href: String,
}

impl HrefElement {
    #[must_use]
    pub const fn new(href: String) -> Self {
        Self { href }
    }
}
