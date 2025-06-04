use derive_more::From;
use rustical_xml::{XmlDeserialize, XmlSerialize};

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
