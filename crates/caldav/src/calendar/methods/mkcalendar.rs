use crate::Error;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use rustical_store::auth::User;
use rustical_store::{Calendar, CalendarStore};
use rustical_xml::{Unparsed, XmlDeserialize, XmlDocument, XmlRootTag};
use tracing::instrument;
use tracing_actix_web::RootSpan;

#[derive(XmlDeserialize, Clone, Debug)]
pub struct MkcolCalendarProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    displayname: Option<String>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    calendar_description: Option<String>,
    #[xml(ns = "rustical_dav::namespace::NS_ICAL")]
    calendar_color: Option<String>,
    #[xml(ns = "rustical_dav::namespace::NS_ICAL")]
    calendar_order: Option<i64>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    calendar_timezone: Option<String>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    calendar_timezone_id: Option<String>,
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    #[allow(dead_code)]
    resourcetype: Unparsed,
}

#[derive(XmlDeserialize, Clone, Debug)]
pub struct PropElement {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    prop: MkcolCalendarProp,
}

#[derive(XmlDeserialize, XmlRootTag, Clone, Debug)]
#[xml(root = b"mkcalendar")]
#[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
struct MkcalendarRequest {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    set: PropElement,
}

#[instrument(parent = root_span.id(), skip(store, root_span))]
pub async fn route_mkcalendar<C: CalendarStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    store: Data<C>,
    root_span: RootSpan,
) -> Result<HttpResponse, Error> {
    let (principal, cal_id) = path.into_inner();
    if principal != user.id {
        return Err(Error::Unauthorized);
    }

    let request = MkcalendarRequest::parse_str(&body)?;
    let request = request.set.prop;

    let calendar = Calendar {
        id: cal_id.to_owned(),
        principal: principal.to_owned(),
        order: request.calendar_order.unwrap_or(0),
        displayname: request.displayname,
        timezone: request.calendar_timezone,
        timezone_id: request.calendar_timezone_id,
        color: request.calendar_color,
        description: request.calendar_description,
        deleted_at: None,
        synctoken: 0,
        subscription_url: None,
        push_topic: uuid::Uuid::new_v4().to_string(),
    };

    match store.insert_calendar(calendar).await {
        // The spec says we should return a mkcalendar-response but I don't know what goes into it.
        // However, it works without one but breaks on iPadOS when using an empty one :)
        Ok(()) => Ok(HttpResponse::Created()
            .insert_header(("Cache-Control", "no-cache"))
            .body("")),
        Err(err) => {
            dbg!(err.to_string());
            Err(err.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_mkcalendar() {
        MkcalendarRequest::parse_str(r#"
            <?xml version='1.0' encoding='UTF-8' ?>
            <CAL:mkcalendar xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
                <set>
                    <prop>
                        <resourcetype>
                            <collection />
                            <CAL:calendar />
                        </resourcetype>
                        <displayname>jfs</displayname>
                        <CAL:calendar-description>rggg</CAL:calendar-description>
                        <n0:calendar-color xmlns:n0="http://apple.com/ns/ical/">#FFF8DCFF</n0:calendar-color>
                        <CAL:calendar-timezone-id>Europe/Berlin</CAL:calendar-timezone-id>
                        <CAL:calendar-timezone>BEGIN:VCALENDAR\r\nBEGIN:VTIMEZONE\r\nTZID:Europe/Berlin\r\nLAST-MODIFIED:20240422T053450Z\r\nTZURL:https://www.tzurl.org/zoneinfo/Europe/Berlin\r\nX-LIC-LOCATION:Europe/Berlin\r\nX-PROLEPTIC-TZNAME:LMT\r\nBEGIN:STANDARD\r\nTZNAME:CET\r\nTZOFFSETFROM:+005328\r\nTZOFFSETTO:+0100\r\nDTSTART:18930401T000632\r\nEND:STANDARD\r\nBEGIN:DAYLIGHT\r\nTZNAME:CEST\r\nTZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nDTSTART:19160430T230000\r\nRDATE:19400401T020000\r\nRDATE:19430329T020000\r\nRDATE:19460414T020000\r\nRDATE:19470406T030000\r\nRDATE:19480418T020000\r\nRDATE:19490410T020000\r\nRDATE:19800406T020000\r\nEND:DAYLIGHT\r\nBEGIN:STANDARD\r\nTZNAME:CET\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nDTSTART:19161001T010000\r\nRDATE:19421102T030000\r\nRDATE:19431004T030000\r\nRDATE:19441002T030000\r\nRDATE:19451118T030000\r\nRDATE:19461007T030000\r\nEND:STANDARD\r\nBEGIN:DAYLIGHT\r\nTZNAME:CEST\r\nTZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nDTSTART:19170416T020000\r\nRRULE:FREQ=YEARLY;UNTIL=19180415T010000Z;BYMONTH=4;BYDAY=3MO\r\nEND:DAYLIGHT\r\nBEGIN:STANDARD\r\nTZNAME:CET\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nDTSTART:19170917T030000\r\nRRULE:FREQ=YEARLY;UNTIL=19180916T010000Z;BYMONTH=9;BYDAY=3MO\r\nEND:STANDARD\r\nBEGIN:DAYLIGHT\r\nTZNAME:CEST\r\nTZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nDTSTART:19440403T020000\r\nRRULE:FREQ=YEARLY;UNTIL=19450402T010000Z;BYMONTH=4;BYDAY=1MO\r\nEND:DAYLIGHT\r\nBEGIN:DAYLIGHT\r\nTZNAME:CEMT\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0300\r\nDTSTART:19450524T000000\r\nRDATE:19470511T010000\r\nEND:DAYLIGHT\r\nBEGIN:DAYLIGHT\r\nTZNAME:CEST\r\nTZOFFSETFROM:+0300\r\nTZOFFSETTO:+0200\r\nDTSTART:19450924T030000\r\nRDATE:19470629T030000\r\nEND:DAYLIGHT\r\nBEGIN:STANDARD\r\nTZNAME:CET\r\nTZOFFSETFROM:+0100\r\nTZOFFSETTO:+0100\r\nDTSTART:19460101T000000\r\nRDATE:19800101T000000\r\nEND:STANDARD\r\nBEGIN:STANDARD\r\nTZNAME:CET\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nDTSTART:19471005T030000\r\nRRULE:FREQ=YEARLY;UNTIL=19491002T010000Z;BYMONTH=10;BYDAY=1SU\r\nEND:STANDARD\r\nBEGIN:STANDARD\r\nTZNAME:CET\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nDTSTART:19800928T030000\r\nRRULE:FREQ=YEARLY;UNTIL=19950924T010000Z;BYMONTH=9;BYDAY=-1SU\r\nEND:STANDARD\r\nBEGIN:DAYLIGHT\r\nTZNAME:CEST\r\nTZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nDTSTART:19810329T020000\r\nRRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU\r\nEND:DAYLIGHT\r\nBEGIN:STANDARD\r\nTZNAME:CET\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nDTSTART:19961027T030000\r\nRRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU\r\nEND:STANDARD\r\nEND:VTIMEZONE\r\nEND:VCALENDAR\r\n</CAL:calendar-timezone>
                    </prop>
                </set>
            </CAL:mkcalendar>
    "#).unwrap();
    }
}
