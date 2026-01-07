use super::FilterElement;
use rstest::rstest;
use rustical_ical::CalendarObject;
use rustical_xml::XmlDocument;

const ICS_1: &str = r"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Example Corp.//CalDAV Client//EN
BEGIN:VTIMEZONE
LAST-MODIFIED:20040110T032845Z
TZID:US/Eastern
BEGIN:DAYLIGHT
DTSTART:20000404T020000
RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=4
TZNAME:EDT
TZOFFSETFROM:-0500
TZOFFSETTO:-0400
END:DAYLIGHT
BEGIN:STANDARD
DTSTART:20001026T020000
RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10
TZNAME:EST
TZOFFSETFROM:-0400
TZOFFSETTO:-0500
END:STANDARD
END:VTIMEZONE
BEGIN:VEVENT
ATTENDEE;PARTSTAT=ACCEPTED;ROLE=CHAIR:mailto:cyrus@example.com
ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:lisa@example.com
DTSTAMP:20060206T001220Z
DTSTART;TZID=US/Eastern:20060104T100000
DURATION:PT1H
LAST-MODIFIED:20060206T001330Z
ORGANIZER:mailto:cyrus@example.com
SEQUENCE:1
STATUS:TENTATIVE
SUMMARY:Event #3
UID:DC6C50A017428C5216A2F1CD@example.com
X-ABC-GUID:E1CX5Dr-0007ym-Hz@example.com
END:VEVENT
END:VCALENDAR
";

const FILTER_1: &str = r#"
<?xml version="1.0" encoding="utf-8" ?>
<C:filter xmlns:C="urn:ietf:params:xml:ns:caldav">
    <C:comp-filter name="VCALENDAR">
        <C:comp-filter name="VEVENT">
            <C:prop-filter name="ATTENDEE">
                <C:text-match collation="i;ascii-casemap">mailto:lisa@example.com</C:text-match>
                <C:param-filter name="PARTSTAT">
                    <C:text-match collation="i;ascii-casemap">NEEDS-ACTION</C:text-match>
                </C:param-filter>
            </C:prop-filter>
        </C:comp-filter>
    </C:comp-filter>
</C:filter>
"#;

const FILTER_2: &str = r#"
<?xml version="1.0" encoding="utf-8" ?>
<C:filter xmlns:C="urn:ietf:params:xml:ns:caldav">
    <C:comp-filter name="VCALENDAR">
        <C:comp-filter name="VEVENT">
            <C:prop-filter name="ATTENDEE">
                <C:text-match collation="i;ascii-casemap">mailto:lisa@example.com</C:text-match>
                <C:param-filter name="PARTSTAT">
                    <C:text-match collation="i;ascii-casemap">ACCEPTED</C:text-match>
                </C:param-filter>
            </C:prop-filter>
        </C:comp-filter>
    </C:comp-filter>
</C:filter>
"#;

#[rstest]
#[case(ICS_1, FILTER_1, true)]
#[case(ICS_1, FILTER_2, false)]
fn yeet(#[case] ics: &str, #[case] filter: &str, #[case] matches: bool) {
    let obj = CalendarObject::from_ics(ics.to_owned(), None).unwrap();
    let filter = FilterElement::parse_str(filter).unwrap();
    assert_eq!(matches, filter.matches(obj.get_inner()));
}
