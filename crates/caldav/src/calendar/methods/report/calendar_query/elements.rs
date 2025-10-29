use crate::calendar_object::CalendarObjectPropWrapperName;
use rustical_dav::xml::PropfindType;
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

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct TextMatchElement {
    #[xml(ty = "attr")]
    pub(crate) collation: String,
    #[xml(ty = "attr")]
    // "yes" or "no", default: "no"
    pub(crate) negate_condition: Option<String>,
}

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

impl CompFilterElement {
    // match the VCALENDAR part
    pub fn matches_root(&self, cal_object: &CalendarObject) -> bool {
        let comp_vcal = self.name == "VCALENDAR";
        match (self.is_not_defined, comp_vcal) {
            // Client wants VCALENDAR to not exist but we are a VCALENDAR
            (Some(()), true) |
            // Client is asking for something different than a vcalendar
            (None, false) => return false,
            _ => {}
        }

        if self.time_range.is_some() {
            // <time-range> should be applied on VEVENT/VTODO but not on VCALENDAR
            return false;
        }

        // TODO: Implement prop-filter at some point

        // Apply sub-comp-filters on VEVENT/VTODO/VJOURNAL component
        if self
            .comp_filter
            .iter()
            .all(|filter| filter.matches(cal_object))
        {
            return true;
        }

        false
    }

    // match the VEVENT/VTODO/VJOURNAL part
    pub fn matches(&self, cal_object: &CalendarObject) -> bool {
        let comp_name_matches = self.name == cal_object.get_component_name();
        match (self.is_not_defined, comp_name_matches) {
            // Client wants VCALENDAR to not exist but we are a VCALENDAR
            (Some(()), true) |
            // Client is asking for something different than a vcalendar
            (None, false) => return false,
            _ => {}
        }

        // TODO: Implement prop-filter (and comp-filter?) at some point

        if let Some(time_range) = &self.time_range {
            if let Some(start) = &time_range.start
                && let Some(last_occurence) = cal_object.get_last_occurence().unwrap_or(None)
                && **start > last_occurence.utc()
            {
                return false;
            }
            if let Some(end) = &time_range.end
                && let Some(first_occurence) = cal_object.get_first_occurence().unwrap_or(None)
                && **end < first_occurence.utc()
            {
                return false;
            }
        }
        true
    }
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
    pub fn matches(&self, cal_object: &CalendarObject) -> bool {
        self.comp_filter.matches_root(cal_object)
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
