use actix_web::{http::StatusCode, HttpRequest};
use rustical_dav::{
    methods::propfind::{PropElement, PropfindType},
    resource::HandlePropfind,
    xml::{
        multistatus::{PropstatWrapper, ResponseElement},
        MultistatusElement,
    },
};
use rustical_store::{
    model::calendar::{format_synctoken, parse_synctoken},
    CalendarStore,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
    calendar_object::resource::{CalendarObjectProp, CalendarObjectResource},
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

pub async fn handle_sync_collection<C: CalendarStore + ?Sized>(
    sync_collection: SyncCollectionRequest,
    req: HttpRequest,
    prefix: &str,
    principal: &str,
    cid: &str,
    cal_store: &RwLock<C>,
) -> Result<MultistatusElement<PropstatWrapper<CalendarObjectProp>, String>, Error> {
    let props = match sync_collection.prop {
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

    let old_synctoken = parse_synctoken(&sync_collection.sync_token).unwrap_or(0);
    let (new_events, deleted_events, new_synctoken) = cal_store
        .read()
        .await
        .sync_changes(principal, cid, old_synctoken)
        .await?;

    let mut responses = Vec::new();
    for event in new_events {
        let path = format!("{}/{}", req.path(), event.get_uid());
        responses.push(
            CalendarObjectResource::from(event)
                .propfind(prefix, &path, props.clone())
                .await?,
        );
    }

    for event_uid in deleted_events {
        let path = format!("{}/{}", req.path(), event_uid);
        responses.push(ResponseElement {
            href: path,
            status: Some(format!("HTTP/1.1 {}", StatusCode::NOT_FOUND)),
            ..Default::default()
        });
    }

    Ok(MultistatusElement {
        responses,
        sync_token: Some(format_synctoken(new_synctoken)),
        ..Default::default()
    })
}
