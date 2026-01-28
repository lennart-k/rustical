use super::{ResponseExtractString, get_app};
use axum::body::Body;
use axum::extract::Request;
use headers::{Authorization, HeaderMapExt};
use http::StatusCode;
use rstest::rstest;
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
use tower::ServiceExt;

const ICS_1: &str = include_str!("resources/rfc4791_appb.ics");

const REPORT_7_8_1: &str = r#"
<?xml version="1.0" encoding="utf-8" ?>
<C:calendar-query xmlns:D="DAV:"
                xmlns:C="urn:ietf:params:xml:ns:caldav">
    <D:prop>
        <D:getetag/>
        <C:calendar-data>
            <C:comp name="VCALENDAR">
            <C:prop name="VERSION"/>
            <C:comp name="VEVENT">
                <C:prop name="SUMMARY"/>
                <C:prop name="UID"/>
                <C:prop name="DTSTART"/>
                <C:prop name="DTEND"/>
                <C:prop name="DURATION"/>
                <C:prop name="RRULE"/>
                <C:prop name="RDATE"/>
                <C:prop name="EXRULE"/>
                <C:prop name="EXDATE"/>
                <C:prop name="RECURRENCE-ID"/>
            </C:comp>
            <C:comp name="VTIMEZONE"/>
            </C:comp>
        </C:calendar-data>
    </D:prop>
    <C:filter>
    <C:comp-filter name="VCALENDAR">
        <C:comp-filter name="VEVENT">
        <C:time-range start="20060104T000000Z"
                        end="20060105T000000Z"/>
        </C:comp-filter>
    </C:comp-filter>
    </C:filter>
</C:calendar-query>
"#;

const REPORT_7_8_2: &str = r#"
<?xml version="1.0" encoding="utf-8" ?>
   <C:calendar-query xmlns:D="DAV:"
                     xmlns:C="urn:ietf:params:xml:ns:caldav">
     <D:prop>
       <C:calendar-data>
         <C:limit-recurrence-set start="20060103T000000Z"
                                 end="20060105T000000Z"/>
       </C:calendar-data>
     </D:prop>
     <C:filter>
       <C:comp-filter name="VCALENDAR">
         <C:comp-filter name="VEVENT">
           <C:time-range start="20060103T000000Z"
                         end="20060105T000000Z"/>
         </C:comp-filter>
       </C:comp-filter>
     </C:filter>
   </C:calendar-query>
"#;

const REPORT_7_8_3: &str = r#"
 <?xml version="1.0" encoding="utf-8" ?>
   <C:calendar-query xmlns:D="DAV:"
                     xmlns:C="urn:ietf:params:xml:ns:caldav">
     <D:prop>
       <C:calendar-data>
         <C:expand start="20060103T000000Z"
                   end="20060105T000000Z"/>
       </C:calendar-data>
     </D:prop>
     <C:filter>
       <C:comp-filter name="VCALENDAR">
         <C:comp-filter name="VEVENT">
           <C:time-range start="20060103T000000Z"
                         end="20060105T000000Z"/>
         </C:comp-filter>
       </C:comp-filter>
     </C:filter>
   </C:calendar-query>
"#;

// Adapted from Example 7.8.3 of RFC 4791
// In the RFC the output is wrong since it returns DTSTART in UTC as local time, e.g.
// DTSTART:20060103T170000
// instead of
// DTSTART:20060103T170000Z
// In https://datatracker.ietf.org/doc/html/rfc4791#section-9.6.5
// it is clearly stated that times with timezone information MUST be returned in UTC.
// Also, the RECURRENCE-ID needs to include the TIMEZONE, which is fixed here by converting it to
// UTC
const OUTPUT_7_8_3: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<multistatus xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CARD="urn:ietf:params:xml:ns:carddav" xmlns:CS="http://calendarserver.org/ns/" xmlns:PUSH="https://bitfire.at/webdav-push">
    <response>
        <href>/caldav/principal/user/calendar/abcd2.ics</href>
        <propstat>
            <prop>
                <CAL:calendar-data>BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Example Corp.//CalDAV Client//EN
BEGIN:VEVENT
DTSTAMP:20060206T001121Z
DTSTART:20060103T170000Z
DURATION:PT1H
SUMMARY:Event #2
UID:abcd2
RECURRENCE-ID:20060103T170000Z
END:VEVENT
BEGIN:VEVENT
DTSTAMP:20060206T001121Z
DTSTART:20060104T190000Z
DURATION:PT1H
RECURRENCE-ID:20060104T170000Z
SUMMARY:Event #2 bis
UID:abcd2
END:VEVENT
END:VCALENDAR
</CAL:calendar-data>
            </prop>
            <status>HTTP/1.1 200 OK</status>
        </propstat>
    </response>
    <response>
        <href>/caldav/principal/user/calendar/abcd3.ics</href>
        <propstat>
            <prop>
                <CAL:calendar-data>BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Example Corp.//CalDAV Client//EN
BEGIN:VEVENT
ATTENDEE;PARTSTAT=ACCEPTED;ROLE=CHAIR:mailto:cyrus@example.com
ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:lisa@example.com
DTSTAMP:20060206T001220Z
DTSTART:20060104T150000Z
DURATION:PT1H
LAST-MODIFIED:20060206T001330Z
ORGANIZER:mailto:cyrus@example.com
SEQUENCE:1
STATUS:TENTATIVE
SUMMARY:Event #3
UID:abcd3
X-ABC-GUID:E1CX5Dr-0007ym-Hz@example.com
END:VEVENT
END:VCALENDAR
</CAL:calendar-data>
            </prop>
            <status>HTTP/1.1 200 OK</status>
        </propstat>
    </response>
</multistatus>"#;

#[rstest]
#[case(0, ICS_1, REPORT_7_8_1, None)]
#[case(1, ICS_1, REPORT_7_8_2, None)]
#[case(2, ICS_1, REPORT_7_8_3, Some(OUTPUT_7_8_3))]
#[tokio::test]
async fn test_report(
    #[from(test_store_context)]
    #[future]
    context: TestStoreContext,
    #[case] case: usize,
    #[case] ics: &'static str,
    #[case] report: &'static str,
    #[case] output: Option<&'static str>,
) {
    let context = context.await;
    let app = get_app(context.clone());

    let (principal, addr_id) = ("user", "calendar");
    let url = format!("/caldav/principal/{principal}/{addr_id}");

    let request_template = || {
        Request::builder()
            .method("IMPORT")
            .uri(&url)
            .body(Body::from(ics))
            .unwrap()
    };
    // Try with correct credentials
    let mut request = request_template();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let mut request = Request::builder()
        .method("REPORT")
        .uri(&url)
        .body(Body::from(report))
        .unwrap();
    request
        .headers_mut()
        .typed_insert(Authorization::basic("user", "pass"));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::MULTI_STATUS);
    let body = response.extract_string().await;
    insta::assert_snapshot!(format!("{case}_report_body"), body);
    if let Some(output) = output {
        similar_asserts::assert_eq!(output, body.replace('\r', ""));
    }
}
