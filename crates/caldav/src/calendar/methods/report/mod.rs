use crate::Error;
use actix_web::{
    HttpRequest, Responder,
    web::{Data, Path},
};
use calendar_multiget::{CalendarMultigetRequest, handle_calendar_multiget};
use calendar_query::{CalendarQueryRequest, handle_calendar_query};
use rustical_dav::xml::{
    PropElement, PropfindType, Propname, sync_collection::SyncCollectionRequest,
};
use rustical_store::{CalendarStore, auth::User};
use rustical_xml::{XmlDeserialize, XmlDocument};
use sync_collection::handle_sync_collection;
use tracing::instrument;

mod calendar_multiget;
mod calendar_query;
mod sync_collection;

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub(crate) struct ExpandElement {
    #[xml(ty = "attr")]
    start: String,
    #[xml(ty = "attr")]
    end: String,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub struct CalendarData {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    comp: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    expand: Option<ExpandElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    limit_recurrence_set: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    limit_freebusy_set: Option<()>,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub enum ReportPropName {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarData(CalendarData),
    #[xml(other)]
    Propname(Propname),
}

#[derive(XmlDeserialize, XmlDocument, Clone, Debug, PartialEq)]
pub(crate) enum ReportRequest {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarMultiget(CalendarMultigetRequest),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarQuery(CalendarQueryRequest),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    SyncCollection(SyncCollectionRequest<ReportPropName>),
}

impl ReportRequest {
    fn props(&self) -> Vec<&str> {
        let prop_element = match self {
            ReportRequest::CalendarMultiget(CalendarMultigetRequest { prop, .. }) => prop,
            ReportRequest::CalendarQuery(CalendarQueryRequest { prop, .. }) => prop,
            ReportRequest::SyncCollection(SyncCollectionRequest { prop, .. }) => prop,
        };

        match prop_element {
            PropfindType::Allprop => {
                vec!["allprop"]
            }
            PropfindType::Propname => {
                vec!["propname"]
            }
            PropfindType::Prop(PropElement(prop_tags)) => prop_tags
                .iter()
                .map(|propname| match propname {
                    ReportPropName::Propname(propname) => propname.name.as_str(),
                    ReportPropName::CalendarData(_) => "calendar-data",
                })
                .collect(),
        }
    }
}

#[instrument(skip(req, cal_store))]
pub async fn route_report_calendar<C: CalendarStore>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    req: HttpRequest,
    cal_store: Data<C>,
) -> Result<impl Responder, Error> {
    let (principal, cal_id) = path.into_inner();
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let request = ReportRequest::parse_str(&body)?;
    let props = request.props();

    Ok(match &request {
        ReportRequest::CalendarQuery(cal_query) => {
            handle_calendar_query(
                cal_query,
                &props,
                req,
                &user,
                &principal,
                &cal_id,
                cal_store.as_ref(),
            )
            .await?
        }
        ReportRequest::CalendarMultiget(cal_multiget) => {
            handle_calendar_multiget(
                cal_multiget,
                &props,
                req,
                &user,
                &principal,
                &cal_id,
                cal_store.as_ref(),
            )
            .await?
        }
        ReportRequest::SyncCollection(sync_collection) => {
            handle_sync_collection(
                sync_collection,
                &props,
                req,
                &user,
                &principal,
                &cal_id,
                cal_store.as_ref(),
            )
            .await?
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use calendar_query::{CompFilterElement, FilterElement, TimeRangeElement};
    use rustical_dav::xml::{PropElement, PropfindType, Propname};
    use rustical_store::calendar::UtcDateTime;
    use rustical_xml::ValueDeserialize;

    #[test]
    fn test_xml_calendar_data() {
        let report_request = ReportRequest::parse_str(r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <calendar-multiget xmlns="urn:ietf:params:xml:ns:caldav" xmlns:D="DAV:">
                <D:prop>
                    <D:getetag/>
                    <D:displayname/>
                    <calendar-data>
                        <expand start="20250426T220000Z" end="20250503T220000Z"/>
                    </calendar-data>
                </D:prop>
                <D:href>/caldav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b</D:href>
            </calendar-multiget>
        "#).unwrap();

        assert_eq!(
            report_request,
            ReportRequest::CalendarMultiget(CalendarMultigetRequest {
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(vec![
                    ReportPropName::Propname(Propname{name: "getetag".to_owned(), ns: Some("DAV:".into())}),
                    ReportPropName::Propname(Propname{name: "displayname".to_owned(), ns: Some("DAV:".into())}),
                    ReportPropName::CalendarData(CalendarData { comp: None, expand: Some(ExpandElement { start: "20250426T220000Z".to_owned(), end: "20250503T220000Z".to_owned() }), limit_recurrence_set: None, limit_freebusy_set: None })
                ])),
                href: vec![
                    "/caldav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b".to_owned()
                ]
            })
        )
    }

    #[test]
    fn test_xml_calendar_query() {
        let report_request = ReportRequest::parse_str(
            r#"
            <?xml version='1.0' encoding='UTF-8' ?>
            <CAL:calendar-query xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav">
                <prop>
                    <getetag />
                </prop>
                <CAL:filter>
                    <CAL:comp-filter name="VCALENDAR">
                        <CAL:comp-filter name="VEVENT">
                            <CAL:time-range start="20240924T143437Z" />
                        </CAL:comp-filter>
                    </CAL:comp-filter>
                </CAL:filter>
            </CAL:calendar-query>"#,
        )
        .unwrap();
        assert_eq!(
            report_request,
            ReportRequest::CalendarQuery(CalendarQueryRequest {
                prop: PropfindType::Prop(PropElement(vec![ReportPropName::Propname(Propname {
                    name: "getetag".to_owned(),
                    ns: Some("DAV:".into())
                })])),
                filter: Some(FilterElement {
                    comp_filter: CompFilterElement {
                        is_not_defined: None,
                        time_range: None,
                        prop_filter: vec![],
                        comp_filter: vec![CompFilterElement {
                            is_not_defined: None,
                            time_range: Some(TimeRangeElement {
                                start: Some(
                                    <UtcDateTime as ValueDeserialize>::deserialize(
                                        "20240924T143437Z"
                                    )
                                    .unwrap()
                                ),
                                end: None
                            }),
                            prop_filter: vec![],
                            comp_filter: vec![],
                            name: "VEVENT".to_owned()
                        }],
                        name: "VCALENDAR".to_owned()
                    }
                }),
                timezone: None,
                timezone_id: None,
            })
        )
    }

    #[test]
    fn test_xml_calendar_multiget() {
        let report_request = ReportRequest::parse_str(r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <calendar-multiget xmlns="urn:ietf:params:xml:ns:caldav" xmlns:D="DAV:">
                <D:prop>
                    <D:getetag/>
                    <D:displayname/>
                </D:prop>
                <D:href>/caldav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b</D:href>
            </calendar-multiget>
        "#).unwrap();

        assert_eq!(
            report_request,
            ReportRequest::CalendarMultiget(CalendarMultigetRequest {
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(vec![
                    ReportPropName::Propname(Propname{name: "getetag".to_owned(), ns: Some("DAV:".into())}),
                    ReportPropName::Propname(Propname{name: "displayname".to_owned(), ns: Some("DAV:".into())})
                ])),
                href: vec![
                    "/caldav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b".to_owned()
                ]
            })
        )
    }
}
