use super::ParamFilterElement;
use ical::{parser::Component, property::Property};
use rustical_dav::xml::TextMatchElement;
use rustical_ical::AddressObject;
use rustical_xml::XmlDeserialize;

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
//  <!ELEMENT prop-filter (is-not-defined |
//                          (text-match*, param-filter*))>
//
//  <!ATTLIST prop-filter name CDATA #REQUIRED
//                          test (anyof | allof) "anyof">
//  <!-- name value: a vCard property name (e.g., "NICKNAME")
//      test value:
//          anyof logical OR for text-match/param-filter matches
//          allof logical AND for text-match/param-filter matches -->
pub struct PropFilterElement {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    pub(crate) is_not_defined: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", flatten)]
    pub(crate) text_match: Vec<TextMatchElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", flatten)]
    pub(crate) param_filter: Vec<ParamFilterElement>,

    #[xml(ty = "attr")]
    pub(crate) name: String,

    #[xml(ty = "attr")]
    pub anyof: Option<String>,
    #[xml(ty = "attr")]
    pub allof: Option<String>,
}

impl PropFilterElement {
    pub fn match_component(&self, comp: &impl PropFilterable) -> bool {
        let property = comp.get_property(&self.name);
        let property = match (self.is_not_defined.is_some(), property) {
            // We are the component that's not supposed to be defined
            (true, Some(_))
            // We don't match
            | (false, None) => return false,
            // We shall not be and indeed we aren't
            (true, None) => return true,
            (false, Some(property)) => property
        };

        let allof = match (self.allof.is_some(), self.anyof.is_some()) {
            (true, false) => true,
            (false, _) => false,
            (true, true) => panic!("wat"),
        };

        let text_matches = self
            .text_match
            .iter()
            .map(|text_match| text_match.match_property(property));

        let param_matches = self
            .param_filter
            .iter()
            .map(|param_filter| param_filter.match_property(property));
        let mut matches = text_matches.chain(param_matches);

        if allof {
            matches.all(|a| a)
        } else {
            matches.any(|a| a)
        }
    }
}

pub trait PropFilterable {
    fn get_property(&self, name: &str) -> Option<&Property>;
}

impl PropFilterable for AddressObject {
    fn get_property(&self, name: &str) -> Option<&Property> {
        self.get_vcard().get_property(name)
    }
}
