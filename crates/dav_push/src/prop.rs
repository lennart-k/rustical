use rustical_dav::header::Depth;
use rustical_xml::XmlSerialize;

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub enum Transport {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    WebPush,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct Transports {
    #[xml(flatten, ty = "untagged")]
    #[xml(ns = "crate::namespace::NS_DAVPUSH")]
    transports: Vec<Transport>,
}

impl Default for Transports {
    fn default() -> Self {
        Self {
            transports: vec![Transport::WebPush],
        }
    }
}

#[derive(XmlSerialize, PartialEq, Clone)]
pub struct SupportedTriggers(#[xml(flatten, ty = "untagged")] pub Vec<SupportedTrigger>);

#[derive(XmlSerialize, PartialEq, Clone)]
pub enum SupportedTrigger {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    ContentUpdate(ContentUpdate),
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    PropertyUpdate(PropertyUpdate),
}

#[derive(XmlSerialize, PartialEq, Clone)]
pub struct ContentUpdate(
    #[xml(rename = b"depth", ns = "rustical_dav::namespace::NS_DAV")] pub Depth,
);

#[derive(XmlSerialize, PartialEq, Clone)]
pub struct PropertyUpdate(
    #[xml(rename = b"depth", ns = "rustical_dav::namespace::NS_DAV")] pub Depth,
);
