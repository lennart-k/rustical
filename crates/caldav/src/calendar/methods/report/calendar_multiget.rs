use super::ReportPropName;
use crate::Error;
use actix_web::dev::{Path, ResourceDef};
use rustical_dav::xml::PropfindType;
use rustical_ical::CalendarObject;
use rustical_store::CalendarStore;
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
