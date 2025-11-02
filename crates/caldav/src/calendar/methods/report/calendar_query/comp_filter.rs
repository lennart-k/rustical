use crate::calendar::methods::report::calendar_query::{
    TimeRangeElement,
    prop_filter::{PropFilterElement, PropFilterable},
};
use rustical_ical::{CalendarObject, CalendarObjectComponent, CalendarObjectType};
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

impl CompFilterable for CalendarObject {
    fn get_comp_name(&self) -> &'static str {
        "VCALENDAR"
    }

    fn match_time_range(&self, _time_range: &TimeRangeElement) -> bool {
        // VCALENDAR has no concept of time range
        false
    }

    fn match_subcomponents(&self, comp_filter: &CompFilterElement) -> bool {
        self.get_data().matches(comp_filter)
    }
}

impl CompFilterable for CalendarObjectComponent {
    fn get_comp_name(&self) -> &'static str {
        CalendarObjectType::from(self).as_str()
    }

    fn match_time_range(&self, time_range: &TimeRangeElement) -> bool {
        if let Some(start) = &time_range.start
            && let Some(last_occurence) = self.get_last_occurence().unwrap_or(None)
            && **start > last_occurence.utc()
        {
            return false;
        }
        if let Some(end) = &time_range.end
            && let Some(first_occurence) = self.get_first_occurence().unwrap_or(None)
            && **end < first_occurence.utc()
        {
            return false;
        }
        true
    }

    fn match_subcomponents(&self, _comp_filter: &CompFilterElement) -> bool {
        // TODO: Properly check subcomponents
        true
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use rustical_ical::{CalendarObject, UtcDateTime};

    use crate::calendar::methods::report::calendar_query::{
        CompFilterable, TextMatchElement, TimeRangeElement, comp_filter::CompFilterElement,
        prop_filter::PropFilterElement,
    };

    const ICS: &str = r"BEGIN:VCALENDAR
CALSCALE:GREGORIAN
VERSION:2.0
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
        let object = CalendarObject::from_ics(ICS.to_string(), None).unwrap();

        let comp_filter = CompFilterElement {
            is_not_defined: Some(()),
            name: "VCALENDAR".to_string(),
            time_range: None,
            prop_filter: vec![],
            comp_filter: vec![],
        };
        assert!(!object.matches(&comp_filter), "filter: wants no VCALENDAR");

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
        assert!(!object.matches(&comp_filter), "filter matches VTODO");

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
        assert!(object.matches(&comp_filter), "filter matches VEVENT");

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
                        needle: "2.0".to_string(),
                        collation: None,
                        negate_condition: None,
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
                        collation: None,
                        negate_condition: None,
                        needle: "weekly".to_string(),
                    }),
                    param_filter: vec![],
                }],
                comp_filter: vec![],
            }],
        };
        assert!(
            object.matches(&comp_filter),
            "Some prop filters on VCALENDAR and VEVENT"
        );
    }
    #[test]
    fn test_comp_filter_time_range() {
        let object = CalendarObject::from_ics(ICS.to_string(), None).unwrap();

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
            object.matches(&comp_filter),
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
            !object.matches(&comp_filter),
            "event should not lie in time range"
        );
    }
}
