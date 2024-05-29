use rustical_caldav::calendar::methods::report::CalendarQueryRequest;

const CALENDAR_QUERY: &str = r#"
<calendar-query xmlns="urn:ietf:params:xml:ns:caldav" xmlns:D="DAV:">
    <D:prop>
        <D:getetag />
    </D:prop>
    <filter>
        <comp-filter name="VCALENDAR">
            <comp-filter name="VEVENT">
                <time-range start="20240423T105630Z" end="20240702T105630Z" />
            </comp-filter>
        </comp-filter>
    </filter>
</calendar-query>
"#;

#[test]
fn test_parse_calendar_query() {
    let query: CalendarQueryRequest = quick_xml::de::from_str(CALENDAR_QUERY).unwrap();
    dbg!(query);
}
