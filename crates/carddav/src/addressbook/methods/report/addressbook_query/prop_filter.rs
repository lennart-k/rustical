use super::{Allof, ParamFilterElement};
use caldata::{component::Component, parser::ContentLine};
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
    #[xml(ty = "attr", default = "Default::default")]
    pub test: Allof,

    #[xml(ty = "attr")]
    pub(crate) name: String,
}

impl PropFilterElement {
    #[must_use]
    pub fn match_property(&self, property: &ContentLine) -> bool {
        if self.param_filter.is_empty() && self.text_match.is_empty() {
            // Filter empty
            return true;
        }
        let Allof(allof) = self.test;
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

    pub fn match_component(&self, comp: &impl PropFilterable) -> bool {
        let mut properties = comp.get_named_properties(&self.name);
        if self.is_not_defined.is_some() {
            return properties.next().is_none();
        }

        // The filter matches when one property instance matches
        properties.any(|prop| self.match_property(prop))
    }
}

pub trait PropFilterable {
    fn get_named_properties<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a ContentLine>;
}

impl PropFilterable for AddressObject {
    fn get_named_properties<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a ContentLine> {
        self.get_vcard().get_named_properties(name)
    }
}
