use rustical_ical::CalendarObject;

const MULTI_VEVENT: &str = r#"
BEGIN:VCALENDAR
PRODID:-//Example Corp.//CalDAV Client//EN
VERSION:2.0
BEGIN:VEVENT
UID:2@example.com
SUMMARY:Weekly Meeting
DTSTAMP:20041210T183838Z
DTSTART:20041206T120000Z
DTEND:20041206T130000Z
RRULE:FREQ=WEEKLY
END:VEVENT
BEGIN:VEVENT
UID:2@example.com
SUMMARY:Weekly Meeting
RECURRENCE-ID:20041213T120000Z
DTSTAMP:20041210T183838Z
DTSTART:20041213T130000Z
DTEND:20041213T140000Z
END:VEVENT
END:VCALENDAR
"#;

#[test]
fn parse_calendar_object() {
    let object = CalendarObject::from_ics(MULTI_VEVENT.to_string(), None).unwrap();
    object.get_inner().expand_recurrence(None, None);
}
