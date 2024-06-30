use crate::{event::resource::EventFile, Error};
use actix_web::{
    web::{Data, Path},
    HttpRequest, Responder,
};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_dav::{
    methods::propfind::{PropElement, PropfindType, ServicePrefix},
    namespace::Namespace,
    resource::HandlePropfind,
    xml::MultistatusElement,
};
use rustical_store::event::Event;
use rustical_store::CalendarStore;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PropQuery {
    Allprop,
    Prop,
    Propname,
}

// TODO: Implement all the other filters

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
// <!ELEMENT calendar-query ((DAV:allprop | DAV:propname | DAV:prop)?, href+)>
pub struct CalendarMultigetRequest {
    #[serde(flatten)]
    prop: PropfindType,
    href: Vec<String>,
}

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

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
struct FilterElement {
    comp_filter: CompFilterElement,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
// #[serde(rename = "calendar-query")]
// <!ELEMENT calendar-query ((DAV:allprop | DAV:propname | DAV:prop)?, filter, timezone?)>
pub struct CalendarQueryRequest {
    #[serde(flatten)]
    prop: PropfindType,
    filter: Option<FilterElement>,
    timezone: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ReportRequest {
    CalendarMultiget(CalendarMultigetRequest),
    CalendarQuery(CalendarQueryRequest),
}

async fn get_events_calendar_query<C: CalendarStore + ?Sized>(
    _cal_query: CalendarQueryRequest,
    principal: &str,
    cid: &str,
    store: &RwLock<C>,
) -> Result<Vec<Event>, Error> {
    // TODO: Implement filtering
    Ok(store.read().await.get_events(principal, cid).await?)
}

async fn get_events_calendar_multiget<C: CalendarStore + ?Sized>(
    _cal_query: CalendarMultigetRequest,
    principal: &str,
    cid: &str,
    store: &RwLock<C>,
) -> Result<Vec<Event>, Error> {
    // TODO: proper implementation
    Ok(store.read().await.get_events(principal, cid).await?)
}

pub async fn route_report_calendar<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    auth: AuthInfoExtractor<A>,
    req: HttpRequest,
    cal_store: Data<RwLock<C>>,
    prefix: Data<ServicePrefix>,
) -> Result<impl Responder, Error> {
    let (principal, cid) = path.into_inner();
    if principal != auth.inner.user_id {
        return Err(Error::Unauthorized);
    }

    let request: ReportRequest = quick_xml::de::from_str(&body)?;
    let events = match request.clone() {
        ReportRequest::CalendarQuery(cal_query) => {
            get_events_calendar_query(cal_query, &principal, &cid, &cal_store).await?
        }
        ReportRequest::CalendarMultiget(cal_multiget) => {
            get_events_calendar_multiget(cal_multiget, &principal, &cid, &cal_store).await?
        }
    };

    // TODO: Change this
    let proptag = match request {
        ReportRequest::CalendarQuery(CalendarQueryRequest { prop, .. }) => prop.clone(),
        ReportRequest::CalendarMultiget(CalendarMultigetRequest { prop, .. }) => prop.clone(),
    };
    let props = match proptag {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            // TODO: Implement
            return Err(Error::NotImplemented);
        }
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags.into(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let mut responses = Vec::new();
    for event in events {
        let path = format!("{}/{}", req.path(), event.get_uid());
        responses.push(
            EventFile { event }
                .propfind(&prefix.0, path, props.clone())
                .await?,
        );
    }

    Ok(MultistatusElement {
        responses,
        member_responses: Vec::<String>::new(),
        ns_dav: Namespace::Dav.as_str(),
        ns_caldav: Namespace::CalDAV.as_str(),
        ns_ical: Namespace::ICal.as_str(),
    })
}
