use crate::calendar::methods::report::calendar_query::{
    TimeRangeElement,
    prop_filter::{PropFilterElement, PropFilterable},
};
use ical::{
    component::{CalendarInnerData, IcalAlarm, IcalCalendarObject, IcalEvent, IcalTodo},
    parser::{Component, ical::component::IcalTimeZone},
};
use rustical_xml::XmlDeserialize;

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
// https://datatracker.ietf.org/doc/html/rfc4791#section-9.7.1
pub struct CompFilterElement {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) is_not_defined: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) time_range: Option<TimeRangeElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", flatten)]
    pub(crate) prop_filter: Vec<PropFilterElement>,
    #[allow(clippy::use_self)]
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", flatten)]
    pub(crate) comp_filter: Vec<CompFilterElement>,

    #[xml(ty = "attr")]
    pub(crate) name: String,
}

pub trait CompFilterable: PropFilterable + Sized {
    fn get_comp_name(&self) -> &'static str;

    fn match_time_range(&self, time_range: &TimeRangeElement) -> bool;

    fn match_subcomponents(&self, comp_filter: &CompFilterElement) -> bool;

    // https://datatracker.ietf.org/doc/html/rfc4791#section-9.7.1
    // The scope of the
    // CALDAV:comp-filter XML element is the calendar object when used as
    // a child of the CALDAV:filter XML element.  The scope of the
    // CALDAV:comp-filter XML element is the enclosing calendar component
    // when used as a child of another CALDAV:comp-filter XML element
    fn matches(&self, comp_filter: &CompFilterElement) -> bool {
        let name_matches = self.get_comp_name() == comp_filter.name;
        match (comp_filter.is_not_defined.is_some(), name_matches) {
            // We are the component that's not supposed to be defined
            (true, true)
            // We don't match
            | (false, false) => return false,
            // We shall not be and indeed we aren't
            (true, false) => return true,
            _ => {}
        }

        if let Some(time_range) = comp_filter.time_range.as_ref()
            && !self.match_time_range(time_range)
        {
            return false;
        }

        for prop_filter in &comp_filter.prop_filter {
            if !prop_filter.match_component(self) {
                return false;
            }
        }

        comp_filter
            .comp_filter
            .iter()
            .all(|filter| self.match_subcomponents(filter))
    }
}

impl CompFilterable for CalendarInnerData {
    fn get_comp_name(&self) -> &'static str {
        match self {
            Self::Event(main, _) => main.get_comp_name(),
            Self::Journal(main, _) => main.get_comp_name(),
            Self::Todo(main, _) => main.get_comp_name(),
        }
    }

    fn match_time_range(&self, time_range: &TimeRangeElement) -> bool {
        if let Some(start) = &time_range.start
            && let Some(last_end) = self.get_last_occurence()
            && start.to_utc() > last_end.utc()
        {
            return false;
        }
        if let Some(end) = &time_range.end
            && let Some(first_start) = self.get_first_occurence()
            && end.to_utc() < first_start.utc()
        {
            return false;
        }
        true
    }

    fn match_subcomponents(&self, comp_filter: &CompFilterElement) -> bool {
        match self {
            Self::Event(main, overrides) => std::iter::once(main)
                .chain(overrides.iter())
                .flat_map(IcalEvent::get_alarms)
                .any(|alarm| alarm.matches(comp_filter)),
            Self::Todo(main, overrides) => std::iter::once(main)
                .chain(overrides.iter())
                .flat_map(IcalTodo::get_alarms)
                .any(|alarm| alarm.matches(comp_filter)),
            // VJOURNAL has no subcomponents
            Self::Journal(_, _) => comp_filter.is_not_defined.is_some(),
        }
    }
}

impl PropFilterable for IcalAlarm {
    fn get_named_properties<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a ical::property::ContentLine> {
        Component::get_named_properties(self, name)
    }
}

impl CompFilterable for IcalAlarm {
    fn get_comp_name(&self) -> &'static str {
        Component::get_comp_name(self)
    }

    fn match_time_range(&self, _time_range: &TimeRangeElement) -> bool {
        true
    }

    fn match_subcomponents(&self, comp_filter: &CompFilterElement) -> bool {
        comp_filter.is_not_defined.is_some()
    }
}

impl PropFilterable for CalendarInnerData {
    #[allow(refining_impl_trait)]
    fn get_named_properties<'a>(
        &'a self,
        name: &'a str,
    ) -> Box<dyn Iterator<Item = &'a ical::property::ContentLine> + 'a> {
        // TODO: If we were pedantic, we would have to do recurrence expansion first
        // and take into account the overrides :(
        match self {
            Self::Event(main, _) => Box::new(main.get_named_properties(name)),
            Self::Todo(main, _) => Box::new(main.get_named_properties(name)),
            Self::Journal(main, _) => Box::new(main.get_named_properties(name)),
        }
    }
}

impl PropFilterable for IcalCalendarObject {
    fn get_named_properties<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a ical::property::ContentLine> {
        Component::get_named_properties(self, name)
    }
}

impl CompFilterable for IcalCalendarObject {
    fn get_comp_name(&self) -> &'static str {
        Component::get_comp_name(self)
    }

    fn match_time_range(&self, _time_range: &TimeRangeElement) -> bool {
        // VCALENDAR has no concept of time range
        false
    }

    fn match_subcomponents(&self, comp_filter: &CompFilterElement) -> bool {
        let mut matches = self
            .get_vtimezones()
            .values()
            .map(|tz| tz.matches(comp_filter))
            .chain([self.get_inner().matches(comp_filter)]);

        if comp_filter.is_not_defined.is_some() {
            matches.all(|x| !x)
        } else {
            matches.any(|x| x)
        }
    }
}

impl PropFilterable for IcalTimeZone {
    fn get_named_properties<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a ical::property::ContentLine> {
        Component::get_named_properties(self, name)
    }
}

impl CompFilterable for IcalTimeZone {
    fn get_comp_name(&self) -> &'static str {
        Component::get_comp_name(self)
    }
    fn match_time_range(&self, _time_range: &TimeRangeElement) -> bool {
        false
    }

    fn match_subcomponents(&self, comp_filter: &CompFilterElement) -> bool {
        // VTIMEZONE has no subcomponents
        comp_filter.is_not_defined.is_some()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use rustical_dav::xml::{MatchType, NegateCondition, TextCollation, TextMatchElement};
    use rustical_ical::{CalendarObject, UtcDateTime};

    use crate::calendar::methods::report::calendar_query::{
        CompFilterElement, CompFilterable, PropFilterElement, TimeRangeElement,
    };

    const ICS: &str = r"BEGIN:VCALENDAR
CALSCALE:GREGORIAN
VERSION:2.0
PRODID:me
BEGIN:VTIMEZONE
TZID:Europe/Berlin
X-LIC-LOCATION:Europe/Berlin
END:VTIMEZONE

BEGIN:VEVENT
UID:318ec6503573d9576818daf93dac07317058d95c
DTSTAMP:20250502T132758Z
DTSTART;TZID=Europe/Berlin:20250506T090000
DTEND;TZID=Europe/Berlin:20250506T092500
SEQUENCE:2
SUMMARY:weekly stuff
TRANSP:OPAQUE
RRULE:FREQ=WEEKLY;COUNT=4;INTERVAL=2;BYDAY=TU,TH,SU
END:VEVENT
END:VCALENDAR";

    #[test]
    fn test_comp_filter_matching() {
        let object = CalendarObject::from_ics(ICS.to_string()).unwrap();

        let comp_filter = CompFilterElement {
            is_not_defined: Some(()),
            name: "VCALENDAR".to_string(),
            time_range: None,
            prop_filter: vec![],
            comp_filter: vec![],
        };
        assert!(
            !object.get_inner().matches(&comp_filter),
            "filter: wants no VCALENDAR"
        );

        let comp_filter = CompFilterElement {
            is_not_defined: None,
            name: "VCALENDAR".to_string(),
            time_range: None,
            prop_filter: vec![],
            comp_filter: vec![CompFilterElement {
                name: "VTODO".to_string(),
                is_not_defined: None,
                time_range: None,
                prop_filter: vec![],
                comp_filter: vec![],
            }],
        };
        assert!(
            !object.get_inner().matches(&comp_filter),
            "filter matches VTODO"
        );

        let comp_filter = CompFilterElement {
            is_not_defined: None,
            name: "VCALENDAR".to_string(),
            time_range: None,
            prop_filter: vec![],
            comp_filter: vec![CompFilterElement {
                name: "VEVENT".to_string(),
                is_not_defined: None,
                time_range: None,
                prop_filter: vec![],
                comp_filter: vec![],
            }],
        };
        assert!(
            object.get_inner().matches(&comp_filter),
            "filter matches VEVENT"
        );

        let comp_filter = CompFilterElement {
            is_not_defined: None,
            name: "VCALENDAR".to_string(),
            time_range: None,
            prop_filter: vec![
                PropFilterElement {
                    is_not_defined: None,
                    name: "VERSION".to_string(),
                    time_range: None,
                    text_match: Some(TextMatchElement {
                        match_type: MatchType::Contains,
                        needle: "2.0".to_string(),
                        collation: TextCollation::default(),
                        negate_condition: NegateCondition::default(),
                    }),
                    param_filter: vec![],
                },
                PropFilterElement {
                    is_not_defined: Some(()),
                    name: "STUFF".to_string(),
                    time_range: None,
                    text_match: None,
                    param_filter: vec![],
                },
            ],
            comp_filter: vec![CompFilterElement {
                name: "VEVENT".to_string(),
                is_not_defined: None,
                time_range: None,
                prop_filter: vec![PropFilterElement {
                    is_not_defined: None,
                    name: "SUMMARY".to_string(),
                    time_range: None,
                    text_match: Some(TextMatchElement {
                        match_type: MatchType::Contains,
                        collation: TextCollation::default(),
                        negate_condition: NegateCondition(false),
                        needle: "weekly".to_string(),
                    }),
                    param_filter: vec![],
                }],
                comp_filter: vec![],
            }],
        };
        assert!(
            object.get_inner().matches(&comp_filter),
            "Some prop filters on VCALENDAR and VEVENT"
        );
    }
    #[test]
    fn test_comp_filter_time_range() {
        let object = CalendarObject::from_ics(ICS.to_string()).unwrap();

        let comp_filter = CompFilterElement {
            is_not_defined: None,
            name: "VCALENDAR".to_string(),
            time_range: None,
            prop_filter: vec![],
            comp_filter: vec![CompFilterElement {
                name: "VEVENT".to_string(),
                is_not_defined: None,
                time_range: Some(TimeRangeElement {
                    start: Some(UtcDateTime(
                        Utc.with_ymd_and_hms(2025, 4, 1, 0, 0, 0).unwrap(),
                    )),
                    end: Some(UtcDateTime(
                        Utc.with_ymd_and_hms(2025, 8, 1, 0, 0, 0).unwrap(),
                    )),
                }),
                prop_filter: vec![],
                comp_filter: vec![],
            }],
        };
        assert!(
            object.get_inner().matches(&comp_filter),
            "event should lie in time range"
        );

        let comp_filter = CompFilterElement {
            is_not_defined: None,
            name: "VCALENDAR".to_string(),
            time_range: None,
            prop_filter: vec![],
            comp_filter: vec![CompFilterElement {
                name: "VEVENT".to_string(),
                is_not_defined: None,
                time_range: Some(TimeRangeElement {
                    start: Some(UtcDateTime(
                        Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap(),
                    )),
                    end: Some(UtcDateTime(
                        Utc.with_ymd_and_hms(2024, 8, 1, 0, 0, 0).unwrap(),
                    )),
                }),
                prop_filter: vec![],
                comp_filter: vec![],
            }],
        };
        assert!(
            !object.get_inner().matches(&comp_filter),
            "event should not lie in time range"
        );
    }

    #[test]
    fn test_match_timezone() {
        let object = CalendarObject::from_ics(ICS.to_string()).unwrap();

        let comp_filter = CompFilterElement {
            is_not_defined: None,
            name: "VCALENDAR".to_string(),
            time_range: None,
            prop_filter: vec![],
            comp_filter: vec![CompFilterElement {
                name: "VTIMEZONE".to_string(),
                is_not_defined: None,
                time_range: None,
                prop_filter: vec![PropFilterElement {
                    is_not_defined: None,
                    name: "TZID".to_string(),
                    time_range: None,
                    text_match: Some(TextMatchElement {
                        match_type: MatchType::Contains,
                        collation: TextCollation::AsciiCasemap,
                        negate_condition: NegateCondition::default(),
                        needle: "Europe/Berlin".to_string(),
                    }),
                    param_filter: vec![],
                }],
                comp_filter: vec![],
            }],
        };
        assert!(
            object.get_inner().matches(&comp_filter),
            "Timezone should be Europe/Berlin"
        );
    }
}
