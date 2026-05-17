use std::str::FromStr;

use crate::{Error, calendar_object::CalendarObjectPropWrapperName};
use http::Uri;
use rustical_dav::{resolve_child_uri, xml::PropfindType};
use rustical_ical::CalendarObject;
use rustical_store::CalendarStore;
use rustical_xml::XmlDeserialize;

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
// <!ELEMENT calendar-query ((DAV:allprop | DAV:propname | DAV:prop)?, href+)>
pub struct CalendarMultigetRequest {
    #[xml(ty = "untagged")]
    pub(crate) prop: PropfindType<CalendarObjectPropWrapperName>,
    #[xml(flatten)]
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    pub(crate) href: Vec<String>,
}

pub async fn get_objects_calendar_multiget<C: CalendarStore>(
    request: &CalendarMultigetRequest,
    collection_uri: &Uri,
    principal: &str,
    cal_id: &str,
    store: &C,
) -> Result<(Vec<(String, CalendarObject)>, Vec<String>), Error> {
    let mut result = vec![];
    let mut not_found = vec![];

    for href in &request.href {
        let Ok(child_uri) = Uri::from_str(href) else {
            not_found.push(href.clone());
            continue;
        };
        let Some(subpath) = resolve_child_uri(collection_uri, &child_uri) else {
            not_found.push(href.clone());
            continue;
        };
        let [filename] = subpath.as_slice() else {
            not_found.push(href.clone());
            continue;
        };
        let Some(object_id) = filename.strip_suffix(".ics") else {
            not_found.push(href.clone());
            continue;
        };
        match store.get_object(principal, cal_id, object_id, false).await {
            Ok(object) => result.push((object_id.to_owned(), object)),
            Err(rustical_store::Error::NotFound) => not_found.push(href.clone()),
            Err(err) => return Err(err.into()),
        }
    }

    Ok((result, not_found))
}

#[cfg(test)]
mod tests {
    use crate::calendar::methods::report::calendar_multiget::{
        CalendarMultigetRequest, get_objects_calendar_multiget,
    };
    use http::Uri;
    use rustical_ical::CalendarObject;
    use rustical_store::CalendarStore;
    use std::panic;

    struct MockCalStore {
        principal: &'static str,
        cal_id: &'static str,
    }

    #[async_trait::async_trait]
    impl CalendarStore for MockCalStore {
        fn is_read_only(&self, _cal_id: &str) -> bool {
            true
        }

        async fn get_calendar(
            &self,
            _principal: &str,
            _id: &str,
            _show_deleted: bool,
        ) -> Result<rustical_store::Calendar, rustical_store::Error> {
            panic!()
        }
        async fn get_calendars(
            &self,
            _principal: &str,
        ) -> Result<Vec<rustical_store::Calendar>, rustical_store::Error> {
            panic!()
        }

        async fn get_deleted_calendars(
            &self,
            _principal: &str,
        ) -> Result<Vec<rustical_store::Calendar>, rustical_store::Error> {
            panic!()
        }

        async fn update_calendar(
            &self,
            _principal: &str,
            _id: &str,
            _calendar: rustical_store::Calendar,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn insert_calendar(
            &self,
            _calendar: rustical_store::Calendar,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn delete_calendar(
            &self,
            _principal: &str,
            _name: &str,
            _use_trashbin: bool,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn restore_calendar(
            &self,
            _principal: &str,
            _name: &str,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn import_calendar(
            &self,
            _calendar: rustical_store::Calendar,
            _objects: Vec<rustical_ical::CalendarObject>,
            _merge_existing: bool,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn sync_changes(
            &self,
            _principal: &str,
            _cal_id: &str,
            _synctoken: i64,
        ) -> Result<
            (
                Vec<(String, rustical_ical::CalendarObject)>,
                Vec<String>,
                i64,
            ),
            rustical_store::Error,
        > {
            panic!()
        }

        async fn calendar_query(
            &self,
            _principal: &str,
            _cal_id: &str,
            _query: rustical_store::calendar_store::CalendarQuery,
        ) -> Result<Vec<(String, rustical_ical::CalendarObject)>, rustical_store::Error> {
            panic!()
        }

        async fn calendar_metadata(
            &self,
            _principal: &str,
            _cal_id: &str,
        ) -> Result<rustical_store::CollectionMetadata, rustical_store::Error> {
            panic!()
        }

        async fn get_objects(
            &self,
            _principal: &str,
            _cal_id: &str,
        ) -> Result<Vec<(String, rustical_ical::CalendarObject)>, rustical_store::Error> {
            panic!()
        }

        async fn get_object(
            &self,
            principal: &str,
            cal_id: &str,
            _object_id: &str,
            _show_deleted: bool,
        ) -> Result<rustical_ical::CalendarObject, rustical_store::Error> {
            if self.principal != principal || self.cal_id != cal_id {
                return Err(rustical_store::Error::NotFound);
            }
            Ok(CalendarObject::from_ics(
                r"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Example Corp.//CalDAV Client//EN
BEGIN:VEVENT
UID:20010712T182145Z-123401@example.com
DTSTAMP:20060712T182145Z
DTSTART:20060714T170000Z
DTEND:20060715T040000Z
SUMMARY:Bastille Day Party
END:VEVENT
END:VCALENDAR"
                    .to_string(),
            )
            .unwrap())
        }

        async fn put_objects(
            &self,
            _principal: &str,
            _cal_id: &str,
            _objects: Vec<(String, rustical_ical::CalendarObject)>,
            _overwrite: bool,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn put_object(
            &self,
            _principal: &str,
            _cal_id: &str,
            _object_id: &str,
            _object: rustical_ical::CalendarObject,
            _overwrite: bool,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn delete_object(
            &self,
            _principal: &str,
            _cal_id: &str,
            _object_id: &str,
            _use_trashbin: bool,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn restore_object(
            &self,
            _principal: &str,
            _cal_id: &str,
            _object_id: &str,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }
    }

    #[tokio::test]
    #[rstest::rstest]
    #[case("/caldav/principal/user%40example%2Ecom/cal/")]
    #[case("/caldav/principal/user%40example.com/cal/")]
    async fn test_multiget_url_escaping(#[case] request_path: &'static str) {
        let req = CalendarMultigetRequest {
            prop: rustical_dav::xml::PropfindType::Propname,
            href: vec![
                "/caldav/principal/user%40example%2Ecom/cal/hello.ics".to_string(),
                "/caldav/principal/user@example.com/cal/unescaped.ics".to_string(),
                "/caldav/principal/user%40example.com/cal/shouldwork.ics".to_string(),
                "/caldav/principal/user%40example%2Ecom/nocal/hello.ics".to_string(),
            ],
        };

        let (result, not_found) = get_objects_calendar_multiget(
            &req,
            &Uri::from_static(request_path),
            "user@example.com",
            "cal",
            &MockCalStore {
                principal: "user@example.com",
                cal_id: "cal",
            },
        )
        .await
        .unwrap();

        let found: Vec<String> = result.into_iter().map(|(href, _)| href).collect();
        similar_asserts::assert_eq!(
            found,
            vec![
                "hello".to_string(),
                "unescaped".to_string(),
                "shouldwork".to_string()
            ]
        );

        similar_asserts::assert_eq!(
            not_found,
            vec!["/caldav/principal/user%40example%2Ecom/nocal/hello.ics".to_string()]
        );
    }
}
