use actix_web::{http::StatusCode, HttpRequest};
use rustical_dav::{
    methods::propfind::{PropElement, PropfindType},
    resource::Resource,
    xml::{
        multistatus::{PropstatWrapper, ResponseElement},
        MultistatusElement,
    },
};
use rustical_store::{
    synctoken::{format_synctoken, parse_synctoken},
    CalendarStore,
};
use serde::Deserialize;

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
    principal: &str,
    cal_id: &str,
    cal_store: &C,
) -> Result<MultistatusElement<PropstatWrapper<CalendarObjectProp>, String>, Error> {
    let props = match sync_collection.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            vec!["propname".to_owned()]
        }
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags.into_inner(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let old_synctoken = parse_synctoken(&sync_collection.sync_token).unwrap_or(0);
    let (new_objects, deleted_objects, new_synctoken) = cal_store
        .sync_changes(principal, cal_id, old_synctoken)
        .await?;

    let mut responses = Vec::new();
    for object in new_objects {
        let path = CalendarObjectResource::get_url(
            req.resource_map(),
            vec![principal, cal_id, &object.get_id()],
        )
        .unwrap();
        responses.push(CalendarObjectResource::from(object).propfind(
            &path,
            props.clone(),
            req.resource_map(),
        )?);
    }

    for object_id in deleted_objects {
        let path = CalendarObjectResource::get_url(
            req.resource_map(),
            vec![principal, cal_id, &object_id],
        )
        .unwrap();
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
