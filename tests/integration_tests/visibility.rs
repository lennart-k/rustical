//! What sharees see of calendar objects at each visibility level.
use rstest::rstest;
use rustical_store::shared::{CalendarObject, Policy, level_of, view};

const FULL: u8 = 4;

/// Every field a client may put on an event carries a marker; the test checks which markers
/// survive a level. (Other subcomponents such as VLOCATION are refused by the parser, so they
/// cannot be stored.)
pub fn canary_event(class: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//test//EN\r
CALSCALE:GREGORIAN\r
METHOD:MARKER-METHOD\r
X-WR-CALNAME:MARKER-CALNAME\r
BEGIN:VTIMEZONE\r
TZID:Europe/Berlin\r
BEGIN:STANDARD\r
DTSTART:19701025T030000\r
TZOFFSETFROM:+0200\r
TZOFFSETTO:+0100\r
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU\r
END:STANDARD\r
END:VTIMEZONE\r
BEGIN:VTIMEZONE\r
TZID:MARKER-UNUSED-TZ\r
BEGIN:STANDARD\r
DTSTART:19701025T030000\r
TZOFFSETFROM:+0000\r
TZOFFSETTO:+0000\r
END:STANDARD\r
END:VTIMEZONE\r
BEGIN:VEVENT\r
UID:canary-uid\r
DTSTAMP:20260101T000000Z\r
CREATED:20260101T000000Z\r
LAST-MODIFIED:20260101T000000Z\r
SEQUENCE:1\r
DTSTART;TZID=Europe/Berlin;X-MARKER=MARKER-PARAM:20260105T100000\r
DTEND;TZID=Europe/Berlin:20260105T110000\r
RRULE:FREQ=WEEKLY;COUNT=3\r
STATUS:CONFIRMED\r
TRANSP:OPAQUE\r
{class}SUMMARY;LANGUAGE=MARKER-LANG:MARKER-SUMMARY\r
LOCATION;ALTREP=\"http://MARKER-ALTREP\":MARKER-LOCATION\r
GEO:48.1;11.5\r
DESCRIPTION:MARKER-DESCRIPTION\r
ATTACH:http://example.com/MARKER-ATTACH\r
ATTENDEE;CN=MARKER-ATTENDEE-CN:mailto:attendee@example.com\r
ORGANIZER;CN=MARKER-ORGANIZER:mailto:MARKER-ORGANIZER-MAIL@example.com\r
CATEGORIES:MARKER-CATEGORIES\r
URL:http://example.com/MARKER-URL\r
CONFERENCE;VALUE=URI:https://example.com/MARKER-CONFERENCE\r
COMMENT:MARKER-COMMENT\r
CONTACT:MARKER-CONTACT\r
X-CUSTOM:MARKER-XPROP\r
BEGIN:VALARM\r
ACTION:DISPLAY\r
DESCRIPTION:MARKER-ALARM\r
TRIGGER:-PT15M\r
END:VALARM\r
END:VEVENT\r
END:VCALENDAR\r
"
    )
}

/// Markers a view at `level` may contain, all others must be gone
fn allowed_markers(level: u8) -> &'static [&'static str] {
    match level {
        1 => &[],
        2 => &["MARKER-SUMMARY"],
        3 => &["MARKER-SUMMARY", "MARKER-LOCATION"],
        _ => unreachable!(),
    }
}

const MARKERS: &[&str] = &[
    "MARKER-METHOD",
    "MARKER-CALNAME",
    "MARKER-UNUSED-TZ",
    "MARKER-PARAM",
    "MARKER-LANG",
    "MARKER-SUMMARY",
    "MARKER-ALTREP",
    "MARKER-LOCATION",
    "MARKER-DESCRIPTION",
    "MARKER-ATTACH",
    "MARKER-ATTENDEE-CN",
    "MARKER-ORGANIZER",
    "MARKER-CATEGORIES",
    "MARKER-URL",
    "MARKER-CONFERENCE",
    "MARKER-COMMENT",
    "MARKER-CONTACT",
    "MARKER-XPROP",
    "MARKER-ALARM",
];

fn object(ics: &str) -> CalendarObject {
    CalendarObject::from_ics(ics.to_owned()).unwrap()
}

#[rstest]
#[case(1)]
#[case(2)]
#[case(3)]
fn test_canary_below_full(#[case] level: u8) {
    let object = object(&canary_event("CLASS:PRIVATE\r\n"));
    let seen = view(&object, level).unwrap();
    let ics = seen.get_ics();
    for marker in MARKERS {
        // MARKER-ORGANIZER is a prefix of MARKER-ORGANIZER-MAIL
        let allowed = allowed_markers(level).contains(marker);
        assert_eq!(
            ics.contains(marker),
            allowed,
            "level {level}, {marker}:\n{ics}"
        );
    }
    // What a busy view keeps
    for kept in [
        "UID:canary-uid",
        "DTSTART;TZID=Europe/Berlin:20260105T100000",
        "RRULE:FREQ=WEEKLY;COUNT=3",
        "STATUS:CONFIRMED",
        "TRANSP:OPAQUE",
        "CLASS:PRIVATE",
        "TZID:Europe/Berlin",
    ] {
        assert!(ics.contains(kept), "level {level}, {kept}:\n{ics}");
    }
    insta::assert_snapshot!(format!("canary_level_{level}"), ics);
}

#[test]
fn test_canary_full_drops_only_alarms() {
    let object = object(&canary_event(""));
    let seen = view(&object, FULL).unwrap();
    let ics = seen.get_ics();
    for marker in MARKERS {
        assert_eq!(
            ics.contains(marker),
            *marker != "MARKER-ALARM",
            "{marker}:\n{ics}"
        );
    }
    assert!(view(&object, 0).is_none());
}

fn event(uid: &str, classes: &[&str]) -> String {
    let classes: String = classes.iter().map(|c| format!("CLASS:{c}\r\n")).collect();
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\nUID:{uid}\r\n\
         DTSTAMP:20260101T000000Z\r\nDTSTART:20260105T100000Z\r\nDTEND:20260105T110000Z\r\n\
         {classes}SUMMARY:Title\r\nDESCRIPTION:Text\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

#[rstest]
// no CLASS and PUBLIC are always fully visible
#[case(&[], Policy { private: 1, confidential: 0 }, 4)]
#[case(&["PUBLIC"], Policy { private: 0, confidential: 0 }, 4)]
#[case(&["public"], Policy { private: 0, confidential: 0 }, 4)]
#[case(&["PRIVATE"], Policy { private: 1, confidential: 0 }, 1)]
#[case(&["Private"], Policy { private: 3, confidential: 0 }, 3)]
#[case(&["CONFIDENTIAL"], Policy { private: 1, confidential: 0 }, 0)]
#[case(&["confidential"], Policy { private: 4, confidential: 2 }, 2)]
// an unknown class counts as PRIVATE (RFC 5545)
#[case(&["X-SECRET"], Policy { private: 2, confidential: 0 }, 2)]
// several: the strictest
#[case(&["PUBLIC", "CONFIDENTIAL"], Policy { private: 3, confidential: 1 }, 1)]
#[case(&["PRIVATE", "PUBLIC"], Policy { private: 2, confidential: 0 }, 2)]
fn test_level_of_class(#[case] classes: &[&str], #[case] policy: Policy, #[case] level: u8) {
    assert_eq!(level_of(&object(&event("uid", classes)), policy), level);
}

#[rstest]
#[case("PRIVATE", 1)]
#[case("PRIVATE", 2)]
#[case("PRIVATE", 3)]
#[case("CONFIDENTIAL", 1)]
fn test_task_views(#[case] class: &str, #[case] level: u8) {
    let task = object(&format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VTODO\r\nUID:task\r\n\
         DTSTAMP:20260101T000000Z\r\nDUE:20260105T100000Z\r\nSTATUS:NEEDS-ACTION\r\n\
         CLASS:{class}\r\nSUMMARY:Task title\r\nLOCATION:Room\r\nDESCRIPTION:Secret\r\n\
         PERCENT-COMPLETE:50\r\nEND:VTODO\r\nEND:VCALENDAR\r\n"
    ));
    let seen = view(&task, level).unwrap();
    let ics = seen.get_ics();
    assert!(ics.contains("DUE:20260105T100000Z"));
    assert!(ics.contains("STATUS:NEEDS-ACTION"));
    assert_eq!(ics.contains("Task title"), level >= 2);
    assert_eq!(ics.contains("Room"), level >= 3);
    assert!(!ics.contains("Secret"));
    assert!(!ics.contains("PERCENT-COMPLETE"));
    insta::assert_snapshot!(format!("task_{class}_level_{level}"), ics);
}

#[test]
fn test_journals_only_full() {
    let journal = |class: &str| {
        object(&format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VJOURNAL\r\nUID:j\r\n\
             DTSTAMP:20260101T000000Z\r\nDTSTART:20260105T100000Z\r\n{class}SUMMARY:Note\r\n\
             DESCRIPTION:Diary\r\nEND:VJOURNAL\r\nEND:VCALENDAR\r\n"
        ))
    };
    let policy = Policy {
        private: 3,
        confidential: 1,
    };
    assert_eq!(level_of(&journal(""), policy), FULL);
    assert_eq!(level_of(&journal("CLASS:PRIVATE\r\n"), policy), 0);
    assert!(view(&journal("CLASS:PRIVATE\r\n"), 3).is_none());
    assert!(
        view(&journal(""), FULL)
            .unwrap()
            .get_ics()
            .contains("Diary")
    );
}

#[test]
fn test_series_with_stricter_override() {
    let series = object(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
         BEGIN:VEVENT\r\nUID:series\r\nDTSTAMP:20260101T000000Z\r\nDTSTART:20260105T100000Z\r\n\
         RRULE:FREQ=DAILY;COUNT=5\r\nCLASS:PUBLIC\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:series\r\nDTSTAMP:20260101T000000Z\r\n\
         RECURRENCE-ID:20260107T100000Z\r\nDTSTART:20260107T120000Z\r\nCLASS:CONFIDENTIAL\r\n\
         SUMMARY:Doctor\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
    );
    // One confidential occurrence makes the whole series confidential
    let policy = Policy {
        private: 4,
        confidential: 1,
    };
    assert_eq!(level_of(&series, policy), 1);
    let seen = view(&series, 1).unwrap();
    let ics = seen.get_ics();
    assert!(!ics.contains("Daily"));
    assert!(!ics.contains("Doctor"));
    assert!(ics.contains("RECURRENCE-ID:20260107T100000Z"));
    insta::assert_snapshot!("series_level_1", ics);
}

#[test]
fn test_policy_rules() {
    // Without a setting of its calendar: both hidden
    assert_eq!(Policy::default(), Policy::new(0, 0).unwrap());
    assert!(Policy::new(4, 4).is_some());
    // CONFIDENTIAL never shows more than PRIVATE
    assert!(Policy::new(1, 2).is_none());
    assert!(Policy::new(5, 0).is_none());
}
