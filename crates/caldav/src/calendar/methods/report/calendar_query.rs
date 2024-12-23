use std::ops::Deref;

use actix_web::HttpRequest;
use rustical_dav::{
    resource::{CommonPropertiesProp, EitherProp, Resource},
    xml::{MultistatusElement, PropElement, PropfindType},
};
use rustical_store::{auth::User, calendar::UtcDateTime, CalendarObject, CalendarStore};
use rustical_xml::{Unit, XmlDeserialize};

use crate::{
    calendar_object::resource::{CalendarObjectProp, CalendarObjectResource},
    Error,
};

// TODO: Implement all the other filters

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
    is_not_defined: Option<Unit>,
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
    is_not_defined: Option<Unit>,
    time_range: Option<TimeRangeElement>,
    text_match: Option<TextMatchElement>,
    #[xml(flatten)]
    param_filter: Vec<ParamFilterElement>,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
// https://datatracker.ietf.org/doc/html/rfc4791#section-9.7.1
pub(crate) struct CompFilterElement {
    pub(crate) is_not_defined: Option<Unit>,
    pub(crate) time_range: Option<TimeRangeElement>,
    #[xml(flatten)]
    pub(crate) prop_filter: Vec<PropFilterElement>,
    #[xml(flatten)]
    pub(crate) comp_filter: Vec<CompFilterElement>,

    #[xml(ty = "attr")]
    pub(crate) name: String,
}

impl CompFilterElement {
    // match the VCALENDAR part
    pub fn matches_root(&self, cal_object: &CalendarObject) -> bool {
        //   A CALDAV:comp-filter is said to match if:
        //
        // *  The CALDAV:comp-filter XML element is empty and the calendar
        //    object or calendar component type specified by the "name"
        //    attribute exists in the current scope;
        //
        // or:
        //
        // *  The CALDAV:comp-filter XML element contains a CALDAV:is-not-
        //    defined XML element and the calendar object or calendar
        //    component type specified by the "name" attribute does not exist
        //    in the current scope;

        let is_defined = self.name == "VCALENDAR";
        if self.is_not_defined.is_some() && is_defined {
            return false;
        }
        if !is_defined {
            return false;
        }

        //
        // or:
        //
        // *  The CALDAV:comp-filter XML element contains a CALDAV:time-range
        //    XML element and at least one recurrence instance in the
        //    targeted calendar component is scheduled to overlap the
        //    specified time range, and all specified CALDAV:prop-filter and
        //    CALDAV:comp-filter child XML elements also match the targeted
        //    calendar component;
        //
        // or:
        //
        // *  The CALDAV:comp-filter XML element only contains CALDAV:prop-
        //    filter and CALDAV:comp-filter child XML elements that all match
        //    the targeted calendar component.
        if self
            .comp_filter
            .iter()
            .map(|filter| filter.matches(cal_object))
            .all(|x| x)
        {
            return true;
        }

        false
    }

    // match the VEVENT/VTODO/VJOURNAL part
    pub fn matches(&self, cal_object: &CalendarObject) -> bool {
        // TODO: evaulate prop-filter
        let component_name = cal_object.get_component_name();
        let is_defined = self.name == component_name;
        if self.is_not_defined.is_some() && is_defined {
            return false;
        }
        if !is_defined {
            return false;
        }

        if let Some(time_range) = &self.time_range {
            if let Some(start) = &time_range.start {
                if let Some(first_occurence) = cal_object.get_first_occurence().unwrap_or(None) {
                    if start.deref() > &first_occurence.utc() {
                        return false;
                    }
                };
            }
            if let Some(end) = &time_range.end {
                if let Some(last_occurence) = cal_object.get_last_occurence().unwrap_or(None) {
                    if end.deref() < &last_occurence.utc() {
                        return false;
                    }
                };
            }
        }

        self.name == component_name
    }
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
// https://datatracker.ietf.org/doc/html/rfc4791#section-9.7
pub(crate) struct FilterElement {
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
    pub(crate) filter: Option<FilterElement>,
    pub(crate) timezone: Option<String>,
}

pub async fn get_objects_calendar_query<C: CalendarStore + ?Sized>(
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

pub async fn handle_calendar_query<C: CalendarStore + ?Sized>(
    cal_query: CalendarQueryRequest,
    req: HttpRequest,
    user: &User,
    principal: &str,
    cal_id: &str,
    cal_store: &C,
) -> Result<MultistatusElement<EitherProp<CalendarObjectProp, CommonPropertiesProp>, String>, Error>
{
    let objects = get_objects_calendar_query(&cal_query, principal, cal_id, cal_store).await?;

    let props = match cal_query.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            vec!["propname".to_owned()]
        }
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags
            .into_iter()
            .map(|propname| propname.name)
            .collect(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let mut responses = Vec::new();
    for object in objects {
        let path = CalendarObjectResource::get_url(
            req.resource_map(),
            vec![principal, cal_id, object.get_id()],
        )
        .unwrap();
        responses.push(
            CalendarObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind(&path, props.clone(), user, req.resource_map())?,
        );
    }

    Ok(MultistatusElement {
        responses,
        ..Default::default()
    })
}
