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
    pub fn get_uid(&self) -> &str {
        self.event.get_uid()
    }

    pub fn get_dtstart(&self) -> Result<Option<CalDateTime>, Error> {
        if let Some(dtstart) = self.event.get_dtstart() {
            Ok(Some(CalDateTime::parse_prop(dtstart, &self.timezones)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_dtend(&self) -> Result<Option<CalDateTime>, Error> {
        if let Some(dtend) = self.event.get_dtend() {
            Ok(Some(CalDateTime::parse_prop(dtend, &self.timezones)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_last_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        if self.event.get_rrule().is_some() {
            // TODO: understand recurrence rules
            return Ok(None);
        }

        if let Some(dtend) = self.get_dtend()? {
            return Ok(Some(dtend));
        };

        let duration = self.event.get_duration().unwrap_or(Duration::days(1));

        let first_occurence = self.get_dtstart()?;
        Ok(first_occurence.map(|first_occurence| first_occurence + duration))
    }

    pub fn recurrence_ruleset(&self) -> Result<Option<rrule::RRuleSet>, Error> {
        let dtstart: DateTime<rrule::Tz> = if let Some(dtstart) = self.get_dtstart()? {
            if let Some(dtend) = self.get_dtend()? {
                // DTSTART and DTEND MUST have the same timezone
                assert_eq!(dtstart.timezone(), dtend.timezone());
            }

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
        overrides: &[EventObject],
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
            let dtstart = self.get_dtstart()?.expect("We must have a DTSTART here");
            let computed_duration = self
                .get_dtend()?
                .map(|dtend| dtend.as_datetime().into_owned() - dtstart.as_datetime().into_owned());

            'recurrence: for date in dates {
                let date = CalDateTime::from(date);
                let dateformat = if dtstart.is_date() {
                    date.format_date()
                } else {
                    date.format()
                };

                for _override in overrides {
                    if let Some(override_id) = &_override
                        .event
                        .get_recurrence_id()
                        .as_ref()
                        .expect("overrides have a recurrence id")
                        .value
                        && override_id == &dateformat
                    {
                        // We have an override for this occurence
                        //
                        events.push(_override.event.clone());
                        continue 'recurrence;
                    }
                }

                let mut ev = self.event.clone().mutable();
                ev.remove_property("RRULE");
                ev.remove_property("RDATE");
                ev.remove_property("EXDATE");
                ev.remove_property("EXRULE");
                let dtstart_prop = ev
                    .get_property("DTSTART")
                    .expect("We must have a DTSTART here")
                    .clone();
                ev.remove_property("DTSTART");
                ev.remove_property("DTEND");

                ev.set_property(Property {
                    name: "RECURRENCE-ID".to_string(),
                    value: Some(dateformat.to_owned()),
                    params: None,
                });
                ev.set_property(Property {
                    name: "DTSTART".to_string(),
                    value: Some(dateformat),
                    params: dtstart_prop.params.clone(),
                });
                if let Some(duration) = computed_duration {
                    let dtend = date + duration;
                    let dtendformat = if dtstart.is_date() {
                        dtend.format_date()
                    } else {
                        dtend.format()
                    };
                    ev.set_property(Property {
                        name: "DTEND".to_string(),
                        value: Some(dtendformat),
                        params: dtstart_prop.params,
                    });
                }
                events.push(ev.verify()?);
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
        let event = CalendarObject::from_ics(ICS.to_string()).unwrap();
        let (event, overrides) = if let crate::CalendarObjectComponent::Event(
            main_event,
            overrides,
        ) = event.get_data()
        {
            (main_event, overrides)
        } else {
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
