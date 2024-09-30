use actix_web::HttpRequest;
use rustical_dav::{
    methods::propfind::{PropElement, PropfindType},
    resource::HandlePropfind,
    xml::{multistatus::PropstatWrapper, MultistatusElement},
};
use rustical_store::{model::object::CalendarObject, CalendarStore};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
    calendar_object::resource::{CalendarObjectProp, CalendarObjectResource},
    Error,
};

// TODO: Implement all the other filters

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
struct TimeRangeElement {
    #[serde(rename = "@start")]
    start: Option<String>,
    #[serde(rename = "@end")]
    end: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
struct ParamFilterElement {
    is_not_defined: Option<()>,
    text_match: Option<TextMatchElement>,

    #[serde(rename = "@name")]
    name: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
struct TextMatchElement {
    #[serde(rename = "@collation")]
    collation: String,
    #[serde(rename = "@negate-collation")]
    negate_collation: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
struct PropFilterElement {
    is_not_defined: Option<()>,
    time_range: Option<TimeRangeElement>,
    text_match: Option<TextMatchElement>,
    #[serde(default)]
    param_filter: Vec<ParamFilterElement>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
// https://datatracker.ietf.org/doc/html/rfc4791#section-9.7.1
struct CompFilterElement {
    is_not_defined: Option<()>,
    time_range: Option<TimeRangeElement>,
    #[serde(default)]
    prop_filter: Vec<PropFilterElement>,
    #[serde(default)]
    comp_filter: Vec<CompFilterElement>,

    #[serde(rename = "@name")]
    name: String,
}

impl CompFilterElement {
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
        //
        //

        // The spec seems a bit unclear. But this should be the correct behaviour I assume
        if self.is_not_defined.is_some() {
            return self.name != "VCALENDAR";
        }
        if self.name != "VCALENDAR" {
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

    pub fn matches(&self, cal_object: &CalendarObject) -> bool {
        // TODO: evaulate prop-filter, time-range
        let component_name = cal_object.get_component_name();

        if self.is_not_defined.is_some() {
            return self.name != component_name;
        }
        self.name == component_name
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
// https://datatracker.ietf.org/doc/html/rfc4791#section-9.7
struct FilterElement {
    comp_filter: CompFilterElement,
}

impl FilterElement {
    pub fn matches(&self, cal_object: &CalendarObject) -> bool {
        self.comp_filter.matches_root(cal_object)
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
// <!ELEMENT calendar-query ((DAV:allprop | DAV:propname | DAV:prop)?, filter, timezone?)>
pub struct CalendarQueryRequest {
    #[serde(flatten)]
    pub prop: PropfindType,
    filter: Option<FilterElement>,
    timezone: Option<String>,
}

pub async fn get_events_calendar_query<C: CalendarStore + ?Sized>(
    cal_query: &CalendarQueryRequest,
    principal: &str,
    cid: &str,
    store: &RwLock<C>,
) -> Result<Vec<CalendarObject>, Error> {
    let mut objects = store.read().await.get_objects(principal, cid).await?;
    if let Some(filter) = &cal_query.filter {
        objects.retain(|object| filter.matches(object));
    }
    Ok(objects)
}

pub async fn handle_calendar_query<C: CalendarStore + ?Sized>(
    cal_query: CalendarQueryRequest,
    req: HttpRequest,
    prefix: &str,
    principal: &str,
    cid: &str,
    cal_store: &RwLock<C>,
) -> Result<MultistatusElement<PropstatWrapper<CalendarObjectProp>, String>, Error> {
    let events = get_events_calendar_query(&cal_query, principal, cid, cal_store).await?;

    let props = match cal_query.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            // TODO: Implement
            return Err(Error::NotImplemented);
        }
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags.into_inner(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let mut responses = Vec::new();
    for event in events {
        let path = format!("{}/{}", req.path(), event.get_uid());
        responses.push(
            CalendarObjectResource::from(event)
                .propfind(prefix, &path, props.clone())
                .await?,
        );
    }

    Ok(MultistatusElement {
        responses,
        ..Default::default()
    })
}
