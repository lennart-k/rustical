use crate::{
    CalDavPrincipalUri, Error,
    calendar::CalendarResourceService,
    calendar_object::{
        CalendarObjectPropWrapper, CalendarObjectPropWrapperName, resource::CalendarObjectResource,
    },
};
use axum::{
    Extension,
    extract::{OriginalUri, Path, State},
    response::IntoResponse,
};
use calendar_multiget::{CalendarMultigetRequest, get_objects_calendar_multiget};
use calendar_query::{CalendarQueryRequest, get_objects_calendar_query};
use http::StatusCode;
use rustical_dav::{
    resource::{PrincipalUri, Resource},
    xml::{
        MultistatusElement, PropfindType, multistatus::ResponseElement,
        sync_collection::SyncCollectionRequest,
    },
};
use rustical_ical::CalendarObject;
use rustical_store::{CalendarStore, SubscriptionStore, auth::Principal};
use rustical_xml::{XmlDeserialize, XmlDocument};
use sync_collection::handle_sync_collection;
use tracing::instrument;

mod calendar_multiget;
mod calendar_query;
mod sync_collection;

#[derive(XmlDeserialize, XmlDocument, Clone, Debug, PartialEq)]
pub(crate) enum ReportRequest {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarMultiget(CalendarMultigetRequest),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarQuery(CalendarQueryRequest),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    SyncCollection(SyncCollectionRequest<CalendarObjectPropWrapperName>),
}

impl ReportRequest {
    const fn props(&self) -> &PropfindType<CalendarObjectPropWrapperName> {
        match &self {
            Self::CalendarMultiget(CalendarMultigetRequest { prop, .. }) => prop,
            Self::CalendarQuery(CalendarQueryRequest { prop, .. }) => prop,
            Self::SyncCollection(SyncCollectionRequest { prop, .. }) => prop,
        }
    }
}

fn objects_response(
    objects: Vec<CalendarObject>,
    not_found: Vec<String>,
    path: &str,
    principal: &str,
    puri: &impl PrincipalUri,
    user: &Principal,
    prop: &PropfindType<CalendarObjectPropWrapperName>,
) -> Result<MultistatusElement<CalendarObjectPropWrapper, String>, Error> {
    let mut responses = Vec::new();
    for object in objects {
        let path = format!("{}/{}.ics", path, object.get_id());
        responses.push(
            CalendarObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind(&path, prop, None, puri, user)?,
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

#[instrument(skip(cal_store))]
pub async fn route_report_calendar<C: CalendarStore, S: SubscriptionStore>(
    Path((principal, cal_id)): Path<(String, String)>,
    user: Principal,
    Extension(puri): Extension<CalDavPrincipalUri>,
    State(CalendarResourceService { cal_store, .. }): State<CalendarResourceService<C, S>>,
    OriginalUri(uri): OriginalUri,
    body: String,
) -> Result<impl IntoResponse, Error> {
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let request = ReportRequest::parse_str(&body)?;
    let props = request.props();

    Ok(match &request {
        ReportRequest::CalendarQuery(cal_query) => {
            let objects =
                get_objects_calendar_query(cal_query, &principal, &cal_id, cal_store.as_ref())
                    .await?;
            objects_response(objects, vec![], uri.path(), &principal, &puri, &user, props)?
        }
        ReportRequest::CalendarMultiget(cal_multiget) => {
            let (objects, not_found) = get_objects_calendar_multiget(
                cal_multiget,
                uri.path(),
                &principal,
                &cal_id,
                cal_store.as_ref(),
            )
            .await?;
            objects_response(
                objects,
                not_found,
                uri.path(),
                &principal,
                &puri,
                &user,
                props,
            )?
        }
        ReportRequest::SyncCollection(sync_collection) => {
            handle_sync_collection(
                sync_collection,
                uri.path(),
                &puri,
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
    use crate::calendar_object::{CalendarData, CalendarObjectPropName, ExpandElement};
    use calendar_query::{CompFilterElement, FilterElement, TimeRangeElement};
    use rustical_dav::{extensions::CommonPropertiesPropName, xml::PropElement};
    use rustical_ical::UtcDateTime;
    use rustical_xml::{NamespaceOwned, ValueDeserialize};

    #[test]
    fn test_xml_calendar_data() {
        let report_request = ReportRequest::parse_str(r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <calendar-multiget xmlns="urn:ietf:params:xml:ns:caldav" xmlns:D="DAV:">
                <D:prop>
                    <D:getetag/>
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
                    CalendarObjectPropWrapperName::CalendarObject(CalendarObjectPropName::Getetag),
                    CalendarObjectPropWrapperName::CalendarObject(CalendarObjectPropName::CalendarData(
                        CalendarData { comp: None, expand: Some(ExpandElement {
                        start: <UtcDateTime as ValueDeserialize>::deserialize("20250426T220000Z").unwrap(),
                        end: <UtcDateTime as ValueDeserialize>::deserialize("20250503T220000Z").unwrap(),
                    }), limit_recurrence_set: None, limit_freebusy_set: None }
                    )),
                ], vec![])),
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
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(
                    vec![CalendarObjectPropWrapperName::CalendarObject(
                        CalendarObjectPropName::Getetag
                    ),],
                    vec![]
                )),
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
                    <D:invalid-prop/>
                </D:prop>
                <D:href>/caldav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b</D:href>
            </calendar-multiget>
        "#).unwrap();

        assert_eq!(
            report_request,
            ReportRequest::CalendarMultiget(CalendarMultigetRequest {
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(vec![
                    CalendarObjectPropWrapperName::CalendarObject(CalendarObjectPropName::Getetag),
                    CalendarObjectPropWrapperName::Common(CommonPropertiesPropName::Displayname),
                ], vec![(Some(NamespaceOwned(Vec::from("DAV:"))), "invalid-prop".to_string())])),
                href: vec![
                    "/caldav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b".to_owned()
                ]
            })
        )
    }
}
