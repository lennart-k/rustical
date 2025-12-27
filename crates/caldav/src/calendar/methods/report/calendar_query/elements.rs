use super::comp_filter::{CompFilterElement, CompFilterable};
use crate::calendar_object::CalendarObjectPropWrapperName;
use rustical_dav::xml::{PropfindType, TextMatchElement};
use rustical_ical::{CalendarObject, UtcDateTime};
use rustical_store::calendar_store::CalendarQuery;
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
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) is_not_defined: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) text_match: Option<TextMatchElement>,

    #[xml(ty = "attr")]
    pub(crate) name: String,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
// https://datatracker.ietf.org/doc/html/rfc4791#section-9.7
pub struct FilterElement {
    // This comp-filter matches on VCALENDAR
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) comp_filter: CompFilterElement,
}

impl FilterElement {
    #[must_use]
    pub fn matches(&self, cal_object: &CalendarObject) -> bool {
        cal_object.matches(&self.comp_filter)
    }
}

impl From<&FilterElement> for CalendarQuery {
    fn from(value: &FilterElement) -> Self {
        let comp_filter_vcalendar = &value.comp_filter;
        for comp_filter in &comp_filter_vcalendar.comp_filter {
            // A calendar object cannot contain both VEVENT and VTODO, so we only have to handle
            // whatever we get first
            if matches!(comp_filter.name.as_str(), "VEVENT" | "VTODO")
                && let Some(time_range) = &comp_filter.time_range
            {
                let start = time_range.start.as_ref().map(|start| start.date_naive());
                let end = time_range.end.as_ref().map(|end| end.date_naive());
                return Self {
                    time_start: start,
                    time_end: end,
                };
            }
        }
        Self::default()
    }
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
// <!ELEMENT calendar-query ((DAV:allprop | DAV:propname | DAV:prop)?, filter, timezone?)>
pub struct CalendarQueryRequest {
    #[xml(ty = "untagged")]
    pub prop: PropfindType<CalendarObjectPropWrapperName>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) filter: Option<FilterElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) timezone: Option<String>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) timezone_id: Option<String>,
}

impl From<&CalendarQueryRequest> for CalendarQuery {
    fn from(value: &CalendarQueryRequest) -> Self {
        value.filter.as_ref().map(Self::from).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::calendar::methods::report::calendar_query::{
        CompFilterElement, FilterElement, TimeRangeElement,
    };
    use chrono::{NaiveDate, TimeZone, Utc};
    use rustical_ical::UtcDateTime;
    use rustical_store::calendar_store::CalendarQuery;

    #[test]
    fn test_filter_element_calendar_query() {
        let filter = FilterElement {
            comp_filter: CompFilterElement {
                name: "VCALENDAR".to_string(),
                is_not_defined: None,
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
            },
        };
        let derived_query: CalendarQuery = (&filter).into();
        let query = CalendarQuery {
            time_start: Some(NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
            time_end: Some(NaiveDate::from_ymd_opt(2024, 8, 1).unwrap()),
        };
        assert_eq!(derived_query, query);
    }
}
