use actix_web::HttpRequest;
use rustical_dav::{
    methods::propfind::{PropElement, PropfindType},
    resource::HandlePropfind,
    xml::{multistatus::PropstatWrapper, MultistatusElement},
};
use rustical_store::{event::Event, CalendarStore};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
    event::resource::{EventFile, EventProp},
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
// <!ELEMENT calendar-query ((DAV:allprop | DAV:propname | DAV:prop)?, filter, timezone?)>
pub struct CalendarQueryRequest {
    #[serde(flatten)]
    pub prop: PropfindType,
    filter: Option<FilterElement>,
    timezone: Option<String>,
}

pub async fn get_events_calendar_query<C: CalendarStore + ?Sized>(
    _cal_query: &CalendarQueryRequest,
    principal: &str,
    cid: &str,
    store: &RwLock<C>,
) -> Result<Vec<Event>, Error> {
    // TODO: Implement filtering
    Ok(store.read().await.get_events(principal, cid).await?)
}

pub async fn handle_calendar_query<C: CalendarStore + ?Sized>(
    cal_query: CalendarQueryRequest,
    req: HttpRequest,
    prefix: &str,
    principal: &str,
    cid: &str,
    cal_store: &RwLock<C>,
) -> Result<MultistatusElement<PropstatWrapper<EventProp>, String>, Error> {
    let events = get_events_calendar_query(&cal_query, principal, cid, cal_store).await?;

    let props = match cal_query.prop {
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
            EventFile::from(event)
                .propfind(prefix, path, props.clone())
                .await?,
        );
    }

    Ok(MultistatusElement {
        responses,
        ..Default::default()
    })
}
