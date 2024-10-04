use crate::{
    calendar_object::resource::{CalendarObjectProp, CalendarObjectResource},
    principal::PrincipalResource,
    Error,
};
use actix_web::{
    dev::{Path, ResourceDef},
    HttpRequest,
};
use rustical_dav::{
    methods::propfind::{PropElement, PropfindType},
    resource::Resource,
    xml::{multistatus::PropstatWrapper, MultistatusElement},
};
use rustical_store::{model::object::CalendarObject, CalendarStore};
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

pub async fn get_objects_calendar_multiget<C: CalendarStore + ?Sized>(
    cal_query: &CalendarMultigetRequest,
    principal_url: &str,
    principal: &str,
    cid: &str,
    store: &RwLock<C>,
) -> Result<Vec<CalendarObject>, Error> {
    // TODO: add proper error results for single events
    let resource_def = ResourceDef::prefix(principal_url).join(&ResourceDef::new("/{cid}/{uid}"));

    let mut result = vec![];

    let store = store.read().await;
    for href in &cal_query.href {
        let mut path = Path::new(href.as_str());
        if !resource_def.capture_match_info(&mut path) {
            // TODO: Handle error
            continue;
        };
        if path.get("cid").unwrap() != cid {
            // TODO: Handle error
            continue;
        }
        let uid = path.get("uid").unwrap();
        result.push(store.get_object(principal, cid, uid).await?);
    }

    Ok(result)
}

pub async fn handle_calendar_multiget<C: CalendarStore + ?Sized>(
    cal_multiget: CalendarMultigetRequest,
    req: HttpRequest,
    principal: &str,
    cid: &str,
    cal_store: &RwLock<C>,
) -> Result<MultistatusElement<PropstatWrapper<CalendarObjectProp>, String>, Error> {
    let principal_url = PrincipalResource::get_url(req.resource_map(), vec![principal]).unwrap();
    let objects =
        get_objects_calendar_multiget(&cal_multiget, &principal_url, principal, cid, cal_store)
            .await?;

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
    for object in objects {
        let path = format!("{}/{}", req.path(), object.get_uid());
        responses.push(
            CalendarObjectResource::from(object)
                .propfind(&path, props.clone(), req.resource_map())
                .await?,
        );
    }

    Ok(MultistatusElement {
        responses,
        ..Default::default()
    })
}
