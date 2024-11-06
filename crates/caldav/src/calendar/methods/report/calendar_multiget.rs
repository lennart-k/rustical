use crate::{
    calendar_object::resource::{CalendarObjectProp, CalendarObjectResource},
    principal::PrincipalResource,
    Error,
};
use actix_web::{
    dev::{Path, ResourceDef},
    http::StatusCode,
    HttpRequest,
};
use rustical_dav::{
    resource::{CommonPropertiesProp, EitherProp, Resource},
    xml::{multistatus::ResponseElement, MultistatusElement, PropElement, PropfindType},
};
use rustical_store::{auth::User, CalendarObject, CalendarStore};
use serde::Deserialize;

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
    cal_id: &str,
    store: &C,
) -> Result<(Vec<CalendarObject>, Vec<String>), Error> {
    let resource_def =
        ResourceDef::prefix(principal_url).join(&ResourceDef::new("/{cal_id}/{object_id}"));

    let mut result = vec![];
    let mut not_found = vec![];

    for href in &cal_query.href {
        let mut path = Path::new(href.as_str());
        if !resource_def.capture_match_info(&mut path) {
            not_found.push(href.to_owned());
        };
        if path.get("cal_id").unwrap() != cal_id {
            not_found.push(href.to_owned());
        }
        let object_id = path.get("object_id").unwrap();
        match store.get_object(principal, cal_id, object_id).await {
            Ok(object) => result.push(object),
            Err(rustical_store::Error::NotFound) => not_found.push(href.to_owned()),
            // TODO: Maybe add error handling on a per-object basis
            Err(err) => return Err(err.into()),
        };
    }

    Ok((result, not_found))
}

pub async fn handle_calendar_multiget<C: CalendarStore + ?Sized>(
    cal_multiget: CalendarMultigetRequest,
    req: HttpRequest,
    user: &User,
    principal: &str,
    cal_id: &str,
    cal_store: &C,
) -> Result<MultistatusElement<EitherProp<CalendarObjectProp, CommonPropertiesProp>, String>, Error>
{
    let principal_url = PrincipalResource::get_url(req.resource_map(), vec![principal]).unwrap();
    let (objects, not_found) =
        get_objects_calendar_multiget(&cal_multiget, &principal_url, principal, cal_id, cal_store)
            .await?;

    let props = match cal_multiget.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            vec!["propname".to_owned()]
        }
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags.into_inner(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let mut responses = Vec::new();
    for object in objects {
        let path = format!("{}/{}", req.path(), object.get_id());
        responses.push(
            CalendarObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind(&path, props.clone(), user, req.resource_map())?,
        );
    }

    let not_found_responses = not_found
        .into_iter()
        .map(|path| ResponseElement {
            href: path,
            status: Some(StatusCode::NOT_FOUND),
            ..Default::default()
        })
        .collect();

    Ok(MultistatusElement {
        responses,
        member_responses: not_found_responses,
        ..Default::default()
    })
}
