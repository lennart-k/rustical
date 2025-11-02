use ical::{
    generator::{IcalCalendar, IcalEvent},
    parser::{
        Component,
        ical::component::{IcalJournal, IcalTodo},
    },
    property::Property,
};
use rustical_ical::{CalendarObject, CalendarObjectComponent};
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

        if let Some(_time_range) = &self.time_range {
            // TODO: implement
            return true;
        }

        if let Some(TextMatchElement {
            collation: _collation,
            negate_condition,
            needle,
        }) = &self.text_match
        {
            let mut matches = property
                .value
                .as_ref()
                .is_some_and(|haystack| haystack.contains(needle));
            match negate_condition.as_deref() {
                None | Some("no") => {}
                Some("yes") => {
                    matches = !matches;
                }
                // Invalid value
                _ => return false,
            }

            if !matches {
                return false;
            }
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

impl PropFilterable for CalendarObjectComponent {
    fn get_property(&self, name: &str) -> Option<&Property> {
        match self {
            Self::Event(event, _) => PropFilterable::get_property(&event.event, name),
            Self::Todo(todo, _) => PropFilterable::get_property(todo, name),
            Self::Journal(journal, _) => PropFilterable::get_property(journal, name),
        }
    }
}
