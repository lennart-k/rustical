use crate::{
    event::resource::{EventFile, EventProp},
    Error,
};
use actix_web::HttpRequest;
use rustical_dav::{
    methods::propfind::{PropElement, PropfindType},
    namespace::Namespace,
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
    _cal_query: &CalendarMultigetRequest,
    principal: &str,
    cid: &str,
    store: &RwLock<C>,
) -> Result<Vec<Event>, Error> {
    // TODO: proper implementation
    Ok(store.read().await.get_events(principal, cid).await?)
}

pub async fn handle_calendar_multiget<C: CalendarStore + ?Sized>(
    cal_multiget: CalendarMultigetRequest,
    req: HttpRequest,
    prefix: &str,
    principal: &str,
    cid: &str,
    cal_store: &RwLock<C>,
) -> Result<MultistatusElement<PropstatWrapper<EventProp>, String>, Error> {
    let events = get_events_calendar_multiget(&cal_multiget, principal, cid, cal_store).await?;

    let props = match cal_multiget.prop {
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
                .propfind(prefix, path, props.clone())
                .await?,
        );
    }

    Ok(MultistatusElement {
        responses,
        member_responses: vec![],
        ns_dav: Namespace::Dav.as_str(),
        ns_caldav: Namespace::CalDAV.as_str(),
        ns_ical: Namespace::ICal.as_str(),
    })
}
