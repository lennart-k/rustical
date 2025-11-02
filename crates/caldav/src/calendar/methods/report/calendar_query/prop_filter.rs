use std::collections::HashMap;

use ical::{
    generator::{IcalCalendar, IcalEvent},
    parser::{
        Component,
        ical::component::{IcalJournal, IcalTimeZone, IcalTodo},
    },
    property::Property,
};
use rustical_ical::{CalDateTime, CalendarObject, CalendarObjectComponent, UtcDateTime};
use rustical_xml::XmlDeserialize;

use crate::calendar::methods::report::calendar_query::{
    ParamFilterElement, TextMatchElement, TimeRangeElement,
};

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

        // TODO: param-filter

        true
    }
}

pub trait PropFilterable {
    fn get_property(&self, name: &str) -> Option<&Property>;
}

impl PropFilterable for CalendarObject {
    fn get_property(&self, name: &str) -> Option<&Property> {
        Self::get_property(self, name)
    }
}

impl PropFilterable for IcalEvent {
    fn get_property(&self, name: &str) -> Option<&Property> {
        Component::get_property(self, name)
    }
}

impl PropFilterable for IcalTodo {
    fn get_property(&self, name: &str) -> Option<&Property> {
        Component::get_property(self, name)
    }
}

impl PropFilterable for IcalJournal {
    fn get_property(&self, name: &str) -> Option<&Property> {
        Component::get_property(self, name)
    }
}

impl PropFilterable for IcalCalendar {
    fn get_property(&self, name: &str) -> Option<&Property> {
        Component::get_property(self, name)
    }
}

impl PropFilterable for IcalTimeZone {
    fn get_property(&self, name: &str) -> Option<&Property> {
        Component::get_property(self, name)
    }
}

impl PropFilterable for CalendarObjectComponent {
    fn get_property(&self, name: &str) -> Option<&Property> {
        match self {
            Self::Event(event, _) => PropFilterable::get_property(&event.event, name),
            Self::Todo(todo, _) => PropFilterable::get_property(todo, name),
            Self::Journal(journal, _) => PropFilterable::get_property(journal, name),
        }
    }
}
