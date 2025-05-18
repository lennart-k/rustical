use crate::Error;
use chrono::Duration;
use ical::{
    generator::IcalEvent,
    parser::{Component, ical::component::IcalTimeZone},
    property::Property,
};
use rustical_ical::{
    CalDateTime, ComponentMut, parse_duration,
    rrule::{ParserError, RecurrenceRule},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EventObject {
    pub(crate) event: IcalEvent,
    pub(crate) timezones: HashMap<String, IcalTimeZone>,
    pub(crate) ics: String,
}

impl EventObject {
    pub fn get_first_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        if let Some(dtstart) = self.event.get_property("DTSTART") {
            Ok(CalDateTime::parse_prop(dtstart, &self.timezones)?)
        } else {
            Ok(None)
        }
    }

    pub fn get_last_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        if let Some(_rrule) = self.event.get_property("RRULE") {
            // TODO: understand recurrence rules
            return Ok(None);
        }

        if let Some(dtend) = self.event.get_property("DTEND") {
            return Ok(CalDateTime::parse_prop(dtend, &self.timezones)?);
        };

        let duration = self.get_duration()?.unwrap_or(Duration::days(1));

        let first_occurence = self.get_first_occurence()?;
        Ok(first_occurence.map(|first_occurence| first_occurence + duration))
    }

    pub fn get_duration(&self) -> Result<Option<Duration>, Error> {
        if let Some(Property {
            value: Some(duration),
            ..
        }) = self.event.get_property("DURATION")
        {
            Ok(Some(parse_duration(duration)?))
        } else {
            Ok(None)
        }
    }

    pub fn recurrence_rule(&self) -> Result<Option<RecurrenceRule>, ParserError> {
        let rrule = if let Some(&Property {
            value: Some(rrule), ..
        }) = self.event.get_property("RRULE").as_ref()
        {
            rrule
        } else {
            return Ok(None);
        };
        RecurrenceRule::parse(rrule).map(Some)
    }

    pub fn expand_recurrence(&self) -> Result<Vec<IcalEvent>, Error> {
        if let Some(rrule) = self.recurrence_rule()? {
            let mut events = vec![];
            let first_occurence = self.get_first_occurence()?.unwrap();
            let dates = rrule.between(first_occurence, None, None);

            for date in dates {
                let dtstart_utc = date;
                let mut ev = self.event.clone();
                ev.remove_property("RRULE");
                ev.set_property(Property {
                    name: "RECURRENCE-ID".to_string(),
                    value: Some(dtstart_utc.format()),
                    params: None,
                });
                ev.set_property(Property {
                    name: "DTSTART".to_string(),
                    value: Some(dtstart_utc.format()),
                    params: None,
                });
                if let Some(duration) = self.get_duration()? {
                    ev.set_property(Property {
                        name: "DTEND".to_string(),
                        value: Some((dtstart_utc + duration).format()),
                        params: None,
                    });
                }
                events.push(ev);
            }
            Ok(events)
        } else {
            Ok(vec![self.event.clone()])
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::CalendarObject;

    const ICS: &str = r#"BEGIN:VCALENDAR
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
END:VCALENDAR"#;

    #[test]
    fn test_expand_recurrence() {
        let event = CalendarObject::from_ics(
            "318ec6503573d9576818daf93dac07317058d95c".to_string(),
            ICS.to_string(),
        )
        .unwrap();
        assert_eq!(event.expand_recurrence().unwrap(), "asd".to_string());
    }
}
