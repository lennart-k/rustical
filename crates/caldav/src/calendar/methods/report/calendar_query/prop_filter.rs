use super::{ParamFilterElement, TimeRangeElement};
use ical::{property::ContentLine, types::CalDateTime};
use rustical_dav::xml::TextMatchElement;
use rustical_ical::UtcDateTime;
use rustical_xml::XmlDeserialize;

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
// https://www.rfc-editor.org/rfc/rfc4791#section-9.7.2
pub struct PropFilterElement {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) is_not_defined: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) time_range: Option<TimeRangeElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) text_match: Option<TextMatchElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", flatten)]
    pub(crate) param_filter: Vec<ParamFilterElement>,

    #[xml(ty = "attr")]
    pub(crate) name: String,
}

pub trait PropFilterable {
    fn get_named_properties<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a ContentLine>;
}

impl PropFilterElement {
    #[must_use]
    pub fn match_property(&self, property: &ContentLine) -> bool {
        if let Some(TimeRangeElement { start, end }) = &self.time_range {
            // TODO: Respect timezones
            let Ok(timestamp) = CalDateTime::parse_prop(property, None) else {
                return false;
            };
            let timestamp = timestamp.utc();
            if let Some(UtcDateTime(start)) = start
                && start > &timestamp
            {
                return false;
            }
            if let Some(UtcDateTime(end)) = end
                && end < &timestamp
            {
                return false;
            }
            return true;
        }

        if let Some(text_match) = &self.text_match
            && !text_match.match_property(property)
        {
            return false;
        }

        if !self
            .param_filter
            .iter()
            .all(|param_filter| param_filter.match_property(property))
        {
            return false;
        }

        true
    }

    pub fn match_component(&self, comp: &impl PropFilterable) -> bool {
        let mut properties = comp.get_named_properties(&self.name);
        if self.is_not_defined.is_some() {
            return properties.next().is_none();
        }

        // The filter matches when one property instance matches
        // Example where this matters: We have multiple attendees and want to match one
        properties.any(|prop| self.match_property(prop))
    }
}
