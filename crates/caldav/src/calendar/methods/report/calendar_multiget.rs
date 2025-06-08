use crate::{Error, calendar_object::resource::CalendarObjectPropWrapperName};
use rustical_dav::xml::PropfindType;
use rustical_ical::CalendarObject;
use rustical_store::CalendarStore;
use rustical_xml::XmlDeserialize;

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
// <!ELEMENT calendar-query ((DAV:allprop | DAV:propname | DAV:prop)?, href+)>
pub(crate) struct CalendarMultigetRequest {
    #[xml(ty = "untagged")]
    pub(crate) prop: PropfindType<CalendarObjectPropWrapperName>,
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
    let mut result = vec![];
    let mut not_found = vec![];

    for href in &cal_query.href {
        if let Some(filename) = href.strip_prefix(path) {
            let filename = filename.trim_start_matches("/");
            if let Some(object_id) = filename.strip_suffix(".ics") {
                match store.get_object(principal, cal_id, object_id).await {
                    Ok(object) => result.push(object),
                    Err(rustical_store::Error::NotFound) => not_found.push(href.to_owned()),
                    Err(err) => return Err(err.into()),
                };
            } else {
                not_found.push(href.to_owned());
                continue;
            }
        } else {
            not_found.push(href.to_owned());
            continue;
        }
    }

    Ok((result, not_found))
}
