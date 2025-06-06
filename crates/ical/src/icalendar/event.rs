use crate::Error;
use crate::{CalDateTime, ComponentMut, parse_duration};
use chrono::{DateTime, Duration, Utc};
use ical::{
    generator::IcalEvent,
    parser::{Component, ical::component::IcalTimeZone},
    property::Property,
};
use rrule::{RRule, RRuleSet};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone)]
pub struct EventObject {
    pub(crate) event: IcalEvent,
    pub(crate) timezones: HashMap<String, IcalTimeZone>,
    pub(crate) ics: String,
}

impl EventObject {
    pub fn get_first_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        if let Some(dtstart) = self.event.get_property("DTSTART") {
            Ok(Some(CalDateTime::parse_prop(dtstart, &self.timezones)?))
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
            return Ok(Some(CalDateTime::parse_prop(dtend, &self.timezones)?));
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

    pub fn recurrence_ruleset(&self) -> Result<Option<rrule::RRuleSet>, Error> {
        let dtstart: DateTime<rrule::Tz> = if let Some(dtstart) = self.get_first_occurence()? {
            dtstart
                .as_datetime()
                .with_timezone(&dtstart.timezone().into())
        } else {
            return Ok(None);
        };

        let mut rrule_set = RRuleSet::new(dtstart);

        for prop in &self.event.properties {
            rrule_set = match prop.name.as_str() {
                "RRULE" => {
                    let rrule = RRule::from_str(prop.value.as_ref().ok_or(Error::RRuleError(
                        rrule::ParseError::MissingDateGenerationRules.into(),
                    ))?)?
                    .validate(dtstart)
                    .unwrap();
                    rrule_set.rrule(rrule)
                }
                "RDATE" => {
                    let rdate = CalDateTime::parse_prop(prop, &self.timezones)?.into();
                    rrule_set.rdate(rdate)
                }
                "EXDATE" => {
                    let exdate = CalDateTime::parse_prop(prop, &self.timezones)?.into();
                    rrule_set.exdate(exdate)
                }
                _ => rrule_set,
            }
        }

        Ok(Some(rrule_set))
    }

    pub fn expand_recurrence(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<Vec<IcalEvent>, Error> {
        if let Some(mut rrule_set) = self.recurrence_ruleset()? {
            if let Some(start) = start {
                rrule_set = rrule_set.after(start.with_timezone(&rrule::Tz::UTC));
            }
            if let Some(end) = end {
                rrule_set = rrule_set.before(end.with_timezone(&rrule::Tz::UTC));
            }
            let mut events = vec![];
            let dates = rrule_set.all(2048).dates;

            for date in dates {
                let date = CalDateTime::from(date);
                let mut ev = self.event.clone();
                ev.remove_property("RRULE");
                ev.remove_property("RDATE");
                ev.remove_property("EXDATE");
                ev.remove_property("EXRULE");
                ev.set_property(Property {
                    name: "RECURRENCE-ID".to_string(),
                    value: Some(date.format()),
                    params: None,
                });
                ev.set_property(Property {
                    name: "DTSTART".to_string(),
                    value: Some(date.format()),
                    params: None,
                });
                if let Some(duration) = self.get_duration()? {
                    ev.set_property(Property {
                        name: "DTEND".to_string(),
                        value: Some((date + duration).format()),
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
    use ical::generator::Emitter;

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

    const EXPANDED: [&str; 4] = [
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
DTEND;TZID=Europe/Berlin:20250506T092500\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250506T090000\r
DTSTART:20250506T090000\r
END:VEVENT\r\n",
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
DTEND;TZID=Europe/Berlin:20250506T092500\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250508T090000\r
DTSTART:20250508T090000\r
END:VEVENT\r\n",
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
DTEND;TZID=Europe/Berlin:20250506T092500\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250511T090000\r
DTSTART:20250511T090000\r
END:VEVENT\r\n",
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
DTEND;TZID=Europe/Berlin:20250506T092500\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250520T090000\r
DTSTART:20250520T090000\r
END:VEVENT\r\n",
    ];

    #[test]
    fn test_expand_recurrence() {
        let event = CalendarObject::from_ics(
            "318ec6503573d9576818daf93dac07317058d95c".to_string(),
            ICS.to_string(),
        )
        .unwrap();
        let event = event.event().unwrap();

        let events: Vec<String> = event
            .expand_recurrence(None, None)
            .unwrap()
            .into_iter()
            .map(|event| Emitter::generate(&event))
            .collect();
        assert_eq!(events.as_slice(), &EXPANDED);
    }
}
