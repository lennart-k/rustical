use super::{ParamFilterElement, TimeRangeElement};
use ical::{
    generator::{IcalCalendar, IcalEvent},
    parser::{
        Component,
        ical::component::{IcalJournal, IcalTimeZone, IcalTodo},
    },
    property::Property,
};
use rustical_dav::xml::TextMatchElement;
use rustical_ical::{CalDateTime, CalendarObject, CalendarObjectComponent, UtcDateTime};
use rustical_xml::XmlDeserialize;
use std::collections::HashMap;

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

impl PropFilterElement {
    #[must_use]
    pub fn match_property(&self, property: &Property) -> bool {
        if let Some(TimeRangeElement { start, end }) = &self.time_range {
            // TODO: Respect timezones
            let Ok(timestamp) = CalDateTime::parse_prop(property, &HashMap::default()) else {
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
        let properties = comp.get_named_properties(&self.name);
        if self.is_not_defined.is_some() {
            return properties.is_empty();
        }

        // The filter matches when one property instance matches
        // Example where this matters: We have multiple attendees and want to match one
        properties.iter().any(|prop| self.match_property(prop))
    }
}

pub trait PropFilterable {
    fn get_named_properties(&self, name: &str) -> Vec<&Property>;
}

impl PropFilterable for CalendarObject {
    fn get_named_properties(&self, name: &str) -> Vec<&Property> {
        Self::get_named_properties(self, name)
    }
}

impl PropFilterable for IcalEvent {
    fn get_named_properties(&self, name: &str) -> Vec<&Property> {
        Component::get_named_properties(self, name)
    }
}

impl PropFilterable for IcalTodo {
    fn get_named_properties(&self, name: &str) -> Vec<&Property> {
        Component::get_named_properties(self, name)
    }
}

impl PropFilterable for IcalJournal {
    fn get_named_properties(&self, name: &str) -> Vec<&Property> {
        Component::get_named_properties(self, name)
    }
}

impl PropFilterable for IcalCalendar {
    fn get_named_properties(&self, name: &str) -> Vec<&Property> {
        Component::get_named_properties(self, name)
    }
}

impl PropFilterable for IcalTimeZone {
    fn get_named_properties(&self, name: &str) -> Vec<&Property> {
        Component::get_named_properties(self, name)
    }
}

impl PropFilterable for CalendarObjectComponent {
    fn get_named_properties(&self, name: &str) -> Vec<&Property> {
        match self {
            Self::Event(event, _) => PropFilterable::get_named_properties(&event.event, name),
            Self::Todo(todo, _) => PropFilterable::get_named_properties(todo, name),
            Self::Journal(journal, _) => PropFilterable::get_named_properties(journal, name),
        }
    }
}
