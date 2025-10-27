use rustical_xml::{ValueDeserialize, ValueSerialize, XmlDeserialize, XmlRootTag};

use super::PropfindType;

#[derive(Clone, Debug, PartialEq, Eq)]
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
                return Err(rustical_xml::XmlError::InvalidValue(
                    rustical_xml::ParseValueError::Other("Invalid sync-level".to_owned()),
                ));
            }
        })
    }
}

impl ValueSerialize for SyncLevel {
    fn serialize(&self) -> String {
        match self {
            Self::One => "1",
            Self::Infinity => "Infinity",
        }
        .to_owned()
    }
}

// https://datatracker.ietf.org/doc/html/rfc5323#section-5.17
#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LimitElement {
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub nresults: NresultsElement,
}

impl From<u64> for LimitElement {
    fn from(value: u64) -> Self {
        Self {
            nresults: NresultsElement(value),
        }
    }
}

impl From<LimitElement> for u64 {
    fn from(value: LimitElement) -> Self {
        value.nresults.0
    }
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct NresultsElement(#[xml(ty = "text")] u64);

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq, XmlRootTag)]
// <!ELEMENT sync-collection (sync-token, sync-level, limit?, prop)>
//    <!-- DAV:limit defined in RFC 5323, Section 5.17 -->
//    <!-- DAV:prop defined in RFC 4918, Section 14.18 -->
#[xml(ns = "crate::namespace::NS_DAV", root = "sync-collection")]
pub struct SyncCollectionRequest<PN: XmlDeserialize> {
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub sync_token: String,
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub sync_level: SyncLevel,
    #[xml(ns = "crate::namespace::NS_DAV", ty = "untagged")]
    pub prop: PropfindType<PN>,
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub limit: Option<LimitElement>,
}

#[cfg(test)]
mod tests {
    use crate::xml::{
        PropElement, PropfindType,
        sync_collection::{SyncCollectionRequest, SyncLevel},
    };
    use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlDocument};

    const SYNC_COLLECTION_REQUEST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
    <sync-collection xmlns="DAV:">
        <sync-token />
        <sync-level>1</sync-level>
        <limit>
            <nresults>100</nresults>
        </limit>
        <prop>
            <getetag />
        </prop>
    </sync-collection>
    "#;

    #[derive(XmlDeserialize, PropName, EnumVariants, PartialEq)]
    #[xml(unit_variants_ident = "TestPropName")]
    enum TestProp {
        Getetag(String),
    }

    #[test]
    fn test_parse_sync_collection_request() {
        let request =
            SyncCollectionRequest::<TestPropName>::parse_str(SYNC_COLLECTION_REQUEST).unwrap();
        assert_eq!(
            request,
            SyncCollectionRequest {
                sync_token: String::new(),
                sync_level: SyncLevel::One,
                prop: PropfindType::Prop(PropElement(vec![TestPropName::Getetag], vec![])),
                limit: Some(100.into())
            }
        );
    }
}
