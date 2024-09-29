use crate::{
    event::resource::{EventProp, EventResource},
    Error,
};
use actix_web::{
    dev::{Path, ResourceDef},
    HttpRequest,
};
use rustical_dav::{
    methods::propfind::{PropElement, PropfindType},
    resource::HandlePropfind,
    xml::{multistatus::PropstatWrapper, MultistatusElement},
};
use rustical_store::{event::Event, CalendarStore};
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
// <!ELEMENT calendar-query ((DAV:allprop | DAV:propname | DAV:prop)?, href+)>
pub struct CalendarMultigetRequest {
    #[serde(flatten)]
    prop: PropfindType,
    href: Vec<String>,
}

pub async fn get_events_calendar_multiget<C: CalendarStore + ?Sized>(
    cal_query: &CalendarMultigetRequest,
    prefix: &str,
    principal: &str,
    cid: &str,
    store: &RwLock<C>,
) -> Result<Vec<Event>, Error> {
    // TODO: add proper error results for single events
    let resource_def =
        ResourceDef::prefix(prefix).join(&ResourceDef::new("/user/{principal}/{cid}/{uid}"));

    let mut result = vec![];

    let store = store.read().await;
    for href in &cal_query.href {
        let mut path = Path::new(href.as_str());
        if !resource_def.capture_match_info(&mut path) {
            // TODO: Handle error
            continue;
        };
        if path.get("principal").unwrap() != principal {
            // TODO: Handle error
            continue;
        }
        if path.get("cid").unwrap() != cid {
            // TODO: Handle error
            continue;
        }
        let uid = path.get("uid").unwrap();
        result.push(store.get_event(principal, cid, uid).await?);
    }

    Ok(result)
}

pub async fn handle_calendar_multiget<C: CalendarStore + ?Sized>(
    cal_multiget: CalendarMultigetRequest,
    req: HttpRequest,
    prefix: &str,
    principal: &str,
    cid: &str,
    cal_store: &RwLock<C>,
) -> Result<MultistatusElement<PropstatWrapper<EventProp>, String>, Error> {
    let events =
        get_events_calendar_multiget(&cal_multiget, prefix, principal, cid, cal_store).await?;

    let props = match cal_multiget.prop {
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
            EventResource::from(event)
                .propfind(prefix, &path, props.clone())
                .await?,
        );
    }

    Ok(MultistatusElement {
        responses,
        ..Default::default()
    })
}
