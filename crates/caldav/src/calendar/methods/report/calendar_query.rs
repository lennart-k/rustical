use std::ops::Deref;

use actix_web::HttpRequest;
use rustical_dav::{
    resource::Resource,
    xml::{MultistatusElement, PropElement, PropfindType},
};
use rustical_store::{auth::User, calendar::UtcDateTime, CalendarObject, CalendarStore};
use rustical_xml::XmlDeserialize;

use crate::{
    calendar_object::resource::{CalendarObjectPropWrapper, CalendarObjectResource},
    Error,
};

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub(crate) struct TimeRangeElement {
    #[xml(ty = "attr")]
    pub(crate) start: Option<UtcDateTime>,
    #[xml(ty = "attr")]
    pub(crate) end: Option<UtcDateTime>,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
struct ParamFilterElement {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    is_not_defined: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    text_match: Option<TextMatchElement>,

    #[xml(ty = "attr")]
    name: String,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
struct TextMatchElement {
    #[xml(ty = "attr")]
    collation: String,
    #[xml(ty = "attr")]
    negate_collation: String,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub(crate) struct PropFilterElement {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    is_not_defined: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    time_range: Option<TimeRangeElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    text_match: Option<TextMatchElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", flatten)]
    param_filter: Vec<ParamFilterElement>,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
// https://datatracker.ietf.org/doc/html/rfc4791#section-9.7.1
pub(crate) struct CompFilterElement {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) is_not_defined: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) time_range: Option<TimeRangeElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", flatten)]
    pub(crate) prop_filter: Vec<PropFilterElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", flatten)]
    pub(crate) comp_filter: Vec<CompFilterElement>,

    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", ty = "attr")]
    pub(crate) name: String,
}

impl CompFilterElement {
    // match the VCALENDAR part
    pub fn matches_root(&self, cal_object: &CalendarObject) -> bool {
        let comp_vcal = self.name == "VCALENDAR";
        match (self.is_not_defined, comp_vcal) {
            // Client wants VCALENDAR to not exist but we are a VCALENDAR
            (Some(()), true) => return false,
            // Client is asking for something different than a vcalendar
            (None, false) => return false,
            _ => {}
        };

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
            (Some(()), true) => return false,
            // Client is asking for something different than a vcalendar
            (None, false) => return false,
            _ => {}
        };

        // TODO: Implement prop-filter (and comp-filter?) at some point

        if let Some(time_range) = &self.time_range {
            if let Some(start) = &time_range.start {
                if let Some(last_occurence) = cal_object.get_last_occurence().unwrap_or(None) {
                    if start.deref() > &last_occurence.utc() {
                        return false;
                    }
                };
            }
            if let Some(end) = &time_range.end {
                if let Some(first_occurence) = cal_object.get_first_occurence().unwrap_or(None) {
                    if end.deref() < &first_occurence.utc() {
                        return false;
                    }
                };
            }
        }
        true
    }
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
// https://datatracker.ietf.org/doc/html/rfc4791#section-9.7
pub(crate) struct FilterElement {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) comp_filter: CompFilterElement,
}

impl FilterElement {
    pub fn matches(&self, cal_object: &CalendarObject) -> bool {
        self.comp_filter.matches_root(cal_object)
    }
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
// <!ELEMENT calendar-query ((DAV:allprop | DAV:propname | DAV:prop)?, filter, timezone?)>
pub struct CalendarQueryRequest {
    #[xml(ty = "untagged")]
    pub prop: PropfindType,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) filter: Option<FilterElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) timezone: Option<String>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) timezone_id: Option<String>,
}

pub async fn get_objects_calendar_query<C: CalendarStore>(
    cal_query: &CalendarQueryRequest,
    principal: &str,
    cal_id: &str,
    store: &C,
) -> Result<Vec<CalendarObject>, Error> {
    let mut objects = store.get_objects(principal, cal_id).await?;
    if let Some(filter) = &cal_query.filter {
        objects.retain(|object| filter.matches(object));
    }
    Ok(objects)
}

pub async fn handle_calendar_query<C: CalendarStore>(
    cal_query: CalendarQueryRequest,
    req: HttpRequest,
    user: &User,
    principal: &str,
    cal_id: &str,
    cal_store: &C,
) -> Result<MultistatusElement<CalendarObjectPropWrapper, String>, Error> {
    let objects = get_objects_calendar_query(&cal_query, principal, cal_id, cal_store).await?;

    let props = match cal_query.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            vec!["propname".to_owned()]
        }
        PropfindType::Prop(PropElement(prop_tags)) => {
            prop_tags.into_iter().map(|propname| propname.0).collect()
        }
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let mut responses = Vec::new();
    for object in objects {
        let path = format!("{}/{}", req.path().trim_end_matches('/'), object.get_id());
        responses.push(
            CalendarObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind(&path, &props, user, req.resource_map())?,
        );
    }

    Ok(MultistatusElement {
        responses,
        ..Default::default()
    })
}
