use super::ReportPropName;
use crate::{
    Error,
    calendar_object::resource::{CalendarObjectPropWrapper, CalendarObjectResource},
};
use actix_web::{
    dev::{Path, ResourceDef},
    http::StatusCode,
};
use rustical_dav::{
    resource::{PrincipalUri, Resource},
    xml::{MultistatusElement, PropfindType, multistatus::ResponseElement},
};
use rustical_store::{CalendarObject, CalendarStore, auth::User};
use rustical_xml::XmlDeserialize;

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
// <!ELEMENT calendar-query ((DAV:allprop | DAV:propname | DAV:prop)?, href+)>
pub(crate) struct CalendarMultigetRequest {
    #[xml(ty = "untagged")]
    pub(crate) prop: PropfindType<ReportPropName>,
    #[xml(flatten)]
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    pub(crate) href: Vec<String>,
}

pub async fn get_objects_calendar_multiget<C: CalendarStore>(
    cal_query: &CalendarMultigetRequest,
    path: &str,
    principal: &str,
    cal_id: &str,
    store: &C,
) -> Result<(Vec<CalendarObject>, Vec<String>), Error> {
    let resource_def = ResourceDef::prefix(path).join(&ResourceDef::new("/{object_id}.ics"));

    let mut result = vec![];
    let mut not_found = vec![];

    for href in &cal_query.href {
        let mut path = Path::new(href.as_str());
        if !resource_def.capture_match_info(&mut path) {
            not_found.push(href.to_owned());
            continue;
        };
        let object_id = path.get("object_id").unwrap();
        match store.get_object(principal, cal_id, object_id).await {
            Ok(object) => result.push(object),
            Err(rustical_store::Error::NotFound) => not_found.push(href.to_owned()),
            Err(err) => return Err(err.into()),
        };
    }

    Ok((result, not_found))
}

pub async fn handle_calendar_multiget<C: CalendarStore>(
    cal_multiget: &CalendarMultigetRequest,
    props: &[&str],
    path: &str,
    puri: &impl PrincipalUri,
    user: &User,
    principal: &str,
    cal_id: &str,
    cal_store: &C,
) -> Result<MultistatusElement<CalendarObjectPropWrapper, String>, Error> {
    let (objects, not_found) =
        get_objects_calendar_multiget(cal_multiget, path, principal, cal_id, cal_store).await?;

    let mut responses = Vec::new();
    for object in objects {
        let path = format!("{}/{}.ics", path, object.get_id());
        responses.push(
            CalendarObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind(&path, props, puri, user)?,
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
