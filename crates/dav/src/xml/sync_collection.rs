use rustical_xml::{ValueDeserialize, ValueSerialize, XmlDeserialize};

use super::PropfindType;

#[derive(Clone, Debug, PartialEq)]
pub enum SyncLevel {
    One,
    Infinity,
}

impl ValueDeserialize for SyncLevel {
    fn deserialize(val: &str) -> Result<Self, rustical_xml::XmlError> {
        Ok(match val {
            "1" => Self::One,
            "Infinity" => Self::Infinity,
            _ => {
                return Err(rustical_xml::XmlError::Other(
                    "Invalid sync-level".to_owned(),
                ))
            }
        })
    }
}

impl ValueSerialize for SyncLevel {
    fn serialize(&self) -> String {
        match self {
            SyncLevel::One => "1",
            SyncLevel::Infinity => "Infinity",
        }
        .to_owned()
    }
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
// <!ELEMENT sync-collection (sync-token, sync-level, limit?, prop)>
//    <!-- DAV:limit defined in RFC 5323, Section 5.17 -->
//    <!-- DAV:prop defined in RFC 4918, Section 14.18 -->
#[xml(ns = "crate::namespace::NS_DAV")]
pub struct SyncCollectionRequest {
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub sync_token: String,
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub sync_level: SyncLevel,
    #[xml(ns = "crate::namespace::NS_DAV", ty = "untagged")]
    pub prop: PropfindType,
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub limit: Option<u64>,
}
