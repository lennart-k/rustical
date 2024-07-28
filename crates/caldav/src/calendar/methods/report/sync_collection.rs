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

use crate::{
    event::resource::{EventFile, EventProp},
    Error,
};

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
enum SyncLevel {
    #[serde(rename = "1")]
    One,
    Infinity,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
// <!ELEMENT sync-collection (sync-token, sync-level, limit?, prop)>
//    <!-- DAV:limit defined in RFC 5323, Section 5.17 -->
//    <!-- DAV:prop defined in RFC 4918, Section 14.18 -->
pub struct SyncCollectionRequest {
    sync_token: String,
    sync_level: SyncLevel,
    timezone: Option<String>,
    #[serde(flatten)]
    pub prop: PropfindType,
    limit: Option<u64>,
}

pub async fn get_events_sync_collection<C: CalendarStore + ?Sized>(
    _sync_collection: &SyncCollectionRequest,
    principal: &str,
    cid: &str,
    store: &RwLock<C>,
) -> Result<Vec<Event>, Error> {
    // TODO: proper implementation
    Ok(store.read().await.get_events(principal, cid).await?)
}

pub async fn handle_sync_collection<C: CalendarStore + ?Sized>(
    sync_collection: SyncCollectionRequest,
    req: HttpRequest,
    prefix: &str,
    principal: &str,
    cid: &str,
    cal_store: &RwLock<C>,
) -> Result<MultistatusElement<PropstatWrapper<EventProp>, String>, Error> {
    let events = get_events_sync_collection(&sync_collection, principal, cid, cal_store).await?;

    let props = match sync_collection.prop {
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
