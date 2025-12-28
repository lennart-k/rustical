use crate::CalDateTime;
use crate::Error;
use chrono::{DateTime, Duration, Utc};
use ical::parser::ComponentMut;
use ical::{generator::IcalEvent, parser::Component, property::Property};
use rrule::{RRule, RRuleSet};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Default)]
pub struct EventObject {
    pub event: IcalEvent,
    // If a timezone is None that means that in the VCALENDAR object there's a timezone defined
    // with that name but its not from the Olson DB
    pub timezones: HashMap<String, Option<chrono_tz::Tz>>,
}

impl EventObject {
    #[must_use]
    pub fn get_uid(&self) -> &str {
        self.event.get_uid()
    }

    pub fn get_last_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        if self.event.get_rrule().is_some() {
            // TODO: understand recurrence rules
            return Ok(None);
        }

        if let Some(dtend) = self.get_dtend()? {
            return Ok(Some(dtend));
        }

        let duration = self.event.get_duration().unwrap_or(Duration::days(1));

        let first_occurence = self.get_dtstart()?;
        Ok(first_occurence.map(|first_occurence| first_occurence + duration))
    }
}

#[cfg(test)]
mod tests {
    use crate::CalendarObject;
    use ical::generator::Emitter;

    const ICS: &str = r"BEGIN:VCALENDAR
CALSCALE:GREGORIAN
VERSION:2.0
BEGIN:VTIMEZONE
TZID:Europe/Berlin
X-LIC-LOCATION:Europe/Berlin
END:VTIMEZONE

BEGIN:VEVENT
UID:318ec6503573d9576818daf93dac07317058d95c
DTSTAMP:20250502T132758Z
DTSTART;TZID=Europe/Berlin:20250506T090000
DTEND;TZID=Europe/Berlin:20250506T092500
SEQUENCE:2
SUMMARY:weekly stuff
TRANSP:OPAQUE
RRULE:FREQ=WEEKLY;COUNT=4;INTERVAL=2;BYDAY=TU,TH,SU
END:VEVENT
END:VCALENDAR";

    const EXPANDED: [&str; 4] = [
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250506T090000\r
DTSTART;TZID=Europe/Berlin:20250506T090000\r
DTEND;TZID=Europe/Berlin:20250506T092500\r
END:VEVENT\r\n",
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250508T090000\r
DTSTART;TZID=Europe/Berlin:20250508T090000\r
DTEND;TZID=Europe/Berlin:20250508T092500\r
END:VEVENT\r\n",
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250511T090000\r
DTSTART;TZID=Europe/Berlin:20250511T090000\r
DTEND;TZID=Europe/Berlin:20250511T092500\r
END:VEVENT\r\n",
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250520T090000\r
DTSTART;TZID=Europe/Berlin:20250520T090000\r
DTEND;TZID=Europe/Berlin:20250520T092500\r
END:VEVENT\r\n",
    ];

    #[test]
    fn test_expand_recurrence() {
        let event = CalendarObject::from_ics(ICS.to_string(), None).unwrap();
        let crate::CalendarObjectComponent::Event(event, overrides) = event.get_data() else {
            panic!()
        };

        let events: Vec<String> = event
            .expand_recurrence(None, None, overrides)
            .unwrap()
            .into_iter()
            .map(|event| Emitter::generate(&event))
            .collect();
        assert_eq!(events.as_slice()[0], EXPANDED[0]);
        assert_eq!(events.as_slice(), &EXPANDED);
    }
}
