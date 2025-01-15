use rustical_xml::XmlSerialize;

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub enum Transport {
    #[xml(ns = "crate::namespace::NS_DAVPUSH")]
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
