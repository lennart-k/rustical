use crate::Error;
use actix_web::{
    web::{Data, Path},
    HttpRequest, Responder,
};
use calendar_multiget::{handle_calendar_multiget, CalendarMultigetRequest};
use calendar_query::{handle_calendar_query, CalendarQueryRequest};
use rustical_store::{auth::User, CalendarStore};
use rustical_xml::{XmlDeserialize, XmlDocument};
use sync_collection::{handle_sync_collection, SyncCollectionRequest};
use tracing::instrument;

mod calendar_multiget;
mod calendar_query;
mod sync_collection;

#[derive(XmlDeserialize, XmlDocument, Clone, Debug, PartialEq)]
pub(crate) enum ReportRequest {
    CalendarMultiget(CalendarMultigetRequest),
    CalendarQuery(CalendarQueryRequest),
    SyncCollection(SyncCollectionRequest),
}

#[instrument(skip(req, cal_store))]
pub async fn route_report_calendar<C: CalendarStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    req: HttpRequest,
    cal_store: Data<C>,
) -> Result<impl Responder, Error> {
    let (principal, cal_id) = path.into_inner();
    if principal != user.id {
        return Err(Error::Unauthorized);
    }

    let request = ReportRequest::parse_str(&body)?;

    Ok(match request.clone() {
        ReportRequest::CalendarQuery(cal_query) => {
            handle_calendar_query(
                cal_query,
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
    use calendar_query::{CompFilterElement, FilterElement, TimeRangeElement};
    use rustical_dav::xml::{PropElement, PropfindType, Propname};
    use rustical_store::calendar::UtcDateTime;
    use rustical_xml::Value;

    use super::*;

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
                prop: PropfindType::Prop(PropElement(vec![Propname("getetag".to_owned())])),
                filter: Some(FilterElement {
                    comp_filter: CompFilterElement {
                        is_not_defined: None,
                        time_range: None,
                        prop_filter: vec![],
                        comp_filter: vec![CompFilterElement {
                            is_not_defined: None,
                            time_range: Some(TimeRangeElement {
                                start: Some(
                                    <UtcDateTime as Value>::deserialize("20240924T143437Z")
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
}
