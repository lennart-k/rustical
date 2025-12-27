use crate::{
    address_object::AddressObjectPropWrapperName,
    addressbook::methods::report::addressbook_query::PropFilterElement,
};
use rustical_dav::xml::{PropfindType, TextMatchElement};
use rustical_ical::{AddressObject, UtcDateTime};
use rustical_xml::XmlDeserialize;

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

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
//  <!ELEMENT filter (prop-filter*)>
//  <!ATTLIST filter test (anyof | allof) "anyof">
//  <!-- test value:
//              anyof logical OR for prop-filter matches
//              allof logical AND for prop-filter matches -->
pub struct FilterElement {
    #[xml(ty = "attr")]
    pub anyof: Option<String>,
    #[xml(ty = "attr")]
    pub allof: Option<String>,
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", flatten)]
    pub(crate) prop_filter: Vec<PropFilterElement>,
}

impl FilterElement {
    #[must_use]
    pub fn matches(&self, addr_object: &AddressObject) -> bool {
        let allof = match (self.allof.is_some(), self.anyof.is_some()) {
            (true, false) => true,
            (false, _) => false,
            (true, true) => panic!("wat"),
        };
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
