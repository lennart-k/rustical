use crate::{
    address_object::AddressObjectPropWrapperName,
    addressbook::methods::report::addressbook_query::PropFilterElement,
};
use derive_more::{From, Into};
use ical::property::Property;
use rustical_dav::xml::{PropfindType, TextMatchElement};
use rustical_ical::{AddressObject, UtcDateTime};
use rustical_xml::{ValueDeserialize, XmlDeserialize, XmlRootTag};

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct TimeRangeElement {
    #[xml(ty = "attr")]
    pub(crate) start: Option<UtcDateTime>,
    #[xml(ty = "attr")]
    pub(crate) end: Option<UtcDateTime>,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
// https://www.rfc-editor.org/rfc/rfc4791#section-9.7.3
pub struct ParamFilterElement {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    pub(crate) is_not_defined: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    pub(crate) text_match: Option<TextMatchElement>,

    #[xml(ty = "attr")]
    pub(crate) name: String,
}

impl ParamFilterElement {
    #[must_use]
    pub fn match_property(&self, prop: &Property) -> bool {
        let Some(param) = prop.get_param(&self.name) else {
            return self.is_not_defined.is_some();
        };
        if self.is_not_defined.is_some() {
            return false;
        }

        let Some(text_match) = self.text_match.as_ref() else {
            return true;
        };
        text_match.match_text(param)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default, From, Into)]
pub struct Allof(pub bool);

impl ValueDeserialize for Allof {
    fn deserialize(val: &str) -> Result<Self, rustical_xml::XmlError> {
        Ok(Self(match val {
            "allof" => true,
            "anyof" => false,
            _ => {
                return Err(rustical_xml::XmlError::InvalidVariant(format!(
                    "Invalid test parameter: {val}"
                )));
            }
        }))
    }
}

//  <!ELEMENT filter (prop-filter*)>
//  <!ATTLIST filter test (anyof | allof) "anyof">
//  <!-- test value:
//              anyof logical OR for prop-filter matches
//              allof logical AND for prop-filter matches -->
#[derive(XmlDeserialize, XmlRootTag, Clone, Debug, PartialEq, Eq)]
#[xml(root = "filter", ns = "rustical_dav::namespace::NS_CARDDAV")]
#[allow(dead_code)]
pub struct FilterElement {
    #[xml(ty = "attr", default = "Default::default")]
    pub test: Allof,
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", flatten)]
    pub(crate) prop_filter: Vec<PropFilterElement>,
}

impl FilterElement {
    #[must_use]
    pub fn matches(&self, addr_object: &AddressObject) -> bool {
        let Allof(allof) = self.test;
        let mut results = self
            .prop_filter
            .iter()
            .map(|prop_filter| prop_filter.match_component(addr_object));
        if allof {
            results.all(|x| x)
        } else {
            results.any(|x| x)
        }
    }
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
//  <!ELEMENT addressbook-query ((DAV:allprop |
//                                  DAV:propname |
//                                  DAV:prop)?, filter, limit?)>
pub struct AddressbookQueryRequest {
    #[xml(ty = "untagged")]
    pub prop: PropfindType<AddressObjectPropWrapperName>,
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    pub(crate) filter: FilterElement,
}
