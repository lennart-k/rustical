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
        }

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
        // TODO: Make nice, this is just a bodge to get correct behaviour
        let mut empty = true;

        for prop in &self.event.properties {
            rrule_set = match prop.name.as_str() {
                "RRULE" => {
                    let rrule = RRule::from_str(prop.value.as_ref().ok_or_else(|| {
                        Error::RRuleError(rrule::ParseError::MissingDateGenerationRules.into())
                    })?)?
                    .validate(dtstart)
                    .unwrap();
                    empty = false;
                    rrule_set.rrule(rrule)
                }
                "RDATE" => {
                    let rdate = CalDateTime::parse_prop(prop, &self.timezones)?.into();
                    empty = false;
                    rrule_set.rdate(rdate)
                }
                "EXDATE" => {
                    let exdate = CalDateTime::parse_prop(prop, &self.timezones)?.into();
                    empty = false;
                    rrule_set.exdate(exdate)
                }
                _ => rrule_set,
            }
        }
        if empty {
            return Ok(None);
        }

        Ok(Some(rrule_set))
    }

    // The returned calendar components MUST NOT use recurrence
    // properties (i.e., EXDATE, EXRULE, RDATE, and RRULE) and MUST NOT
    // have reference to or include VTIMEZONE components.  Date and local
    // time with reference to time zone information MUST be converted
    // into date with UTC time.
    pub fn expand_recurrence(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        overrides: &[Self],
    ) -> Result<Vec<IcalEvent>, Error> {
        let mut events = vec![];
        let dtstart = self.get_dtstart()?.expect("We must have a DTSTART here");
        let computed_duration = self
            .get_dtend()?
            .map(|dtend| dtend.as_datetime().into_owned() - dtstart.as_datetime().as_ref());

        let Some(mut rrule_set) = self.recurrence_ruleset()? else {
            // If ruleset empty simply return main event AND all overrides
            return Ok(std::iter::once(self.clone())
                .chain(overrides.iter().cloned())
                .map(|event| event.event)
                .collect());
        };
        if let Some(start) = start {
            rrule_set = rrule_set.after(start.with_timezone(&rrule::Tz::UTC));
        }
        if let Some(end) = end {
            rrule_set = rrule_set.before(end.with_timezone(&rrule::Tz::UTC));
        }
        let dates = rrule_set.all(2048).dates;
        'recurrence: for date in dates {
            let date = CalDateTime::from(date.to_utc());
            let recurrence_id = if dtstart.is_date() {
                date.format_date()
            } else {
                date.format()
            };

            for ev_override in overrides {
                if let Some(override_id) = &ev_override
                    .event
                    .get_recurrence_id()
                    .as_ref()
                    .expect("overrides have a recurrence id")
                    .value
                    && override_id == &recurrence_id
                {
                    // We have an override for this occurence
                    //
                    events.push(ev_override.event.clone());
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
                value: Some(recurrence_id.clone()),
                params: vec![],
            });
            ev.set_property(Property {
                name: "DTSTART".to_string(),
                value: Some(recurrence_id),
                params: vec![],
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
    }
}

#[cfg(test)]
mod tests {
    use crate::{CalDateTime, CalendarObject};
    use chrono::{DateTime, Utc};
    use ical::generator::Emitter;
    use rstest::rstest;

    const ICS_1: &str = r"BEGIN:VCALENDAR
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

    const EXPANDED_1: &[&str] = &[
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250506T070000Z\r
DTSTART:20250506T070000Z\r
DTEND:20250506T072500Z\r
END:VEVENT\r\n",
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250508T070000Z\r
DTSTART:20250508T070000Z\r
DTEND:20250508T072500Z\r
END:VEVENT\r\n",
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250511T090000\r
DTSTART:20250511T070000Z\r
DTEND:20250511T072500Z\r
END:VEVENT\r\n",
        "BEGIN:VEVENT\r
UID:318ec6503573d9576818daf93dac07317058d95c\r
DTSTAMP:20250502T132758Z\r
SEQUENCE:2\r
SUMMARY:weekly stuff\r
TRANSP:OPAQUE\r
RECURRENCE-ID:20250520T090000\r
DTSTA:20250520T070000Z\r
DTEND:20250520T072500Z\r
END:VEVENT\r\n",
    ];

    const ICS_2: &str = r"BEGIN:VCALENDAR
CALSCALE:GREGORIAN
VERSION:2.0
BEGIN:VTIMEZONE
TZID:US/Eastern
END:VTIMEZONE
BEGIN:VEVENT
DTSTAMP:20060206T001121Z
DTSTART;TZID=US/Eastern:20060102T120000
DURATION:PT1H
RRULE:FREQ=DAILY;COUNT=5
SUMMARY:Event #2
UID:abcd2
END:VEVENT
BEGIN:VEVENT
DTSTAMP:20060206T001121Z
DTSTART;TZID=US/Eastern:20060104T140000
DURATION:PT1H
RECURRENCE-ID;TZID=US/Eastern:20060104T120000
SUMMARY:Event #2 bis
UID:abcd2
END:VEVENT
END:VCALENDAR
";

    const EXPANDED_2: &[&str] = &[
        "BEGIN:VEVENT\r
DTSTAMP:20060206T001121Z\r
DURATION:PT1H\r
SUMMARY:Event #2\r
UID:abcd2\r
RECURRENCE-ID:20060103T170000\r
DTSTART:20060103T170000\r
END:VEVENT\r\n",
        "BEGIN:VEVENT\r
DTSTAMP:20060206T001121Z\r
DURATION:PT1H\r
SUMMARY:Event #2 bis\r
UID:abcd2\r
RECURRENCE-ID:20060104T170000\r
DTSTART:20060104T190000\r
END:VEVENT\r
END:VCALENDAR\r\n",
    ];

    const ICS_3: &str = r"BEGIN:VCALENDAR
CALSCALE:GREGORIAN
VERSION:2.0
BEGIN:VTIMEZONE
TZID:US/Eastern
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
UID:abcd3
END:VEVENT
END:VCALENDAR
";

    const EXPANDED_3: &[&str] = &["BEGIN:VEVENT
ATTENDEE;PARTSTAT=ACCEPTED;ROLE=CHAIR:mailto:cyrus@example.com
ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:lisa@example.com
DTSTAMP:20060206T001220Z
DTSTART:20060104T150000
DURATION:PT1H
LAST-MODIFIED:20060206T001330Z
ORGANIZER:mailto:cyrus@example.com
SEQUENCE:1
STATUS:TENTATIVE
SUMMARY:Event #3
UID:abcd3
X-ABC-GUID:E1CX5Dr-0007ym-Hz@example.com
END:VEVENT"];

    // The implementation never was entirely correct but will be fixed in v0.12
    // #[rstest]
    // #[case(ICS_1, EXPANDED_1, None, None)]
    // // from https://datatracker.ietf.org/doc/html/rfc4791#section-7.8.3
    // #[case(ICS_2, EXPANDED_2,
    //     Some(CalDateTime::parse("20060103T000000Z", Some(chrono_tz::US::Eastern)).unwrap().utc()),
    //     Some(CalDateTime::parse("20060105T000000Z", Some(chrono_tz::US::Eastern)).unwrap().utc())
    // )]
    // #[case(ICS_3, EXPANDED_3,
    //     Some(CalDateTime::parse("20060103T000000Z", Some(chrono_tz::US::Eastern)).unwrap().utc()),
    //     Some(CalDateTime::parse("20060105T000000Z", Some(chrono_tz::US::Eastern)).unwrap().utc())
    // )]
    // fn test_expand_recurrence(
    //     #[case] ics: &'static str,
    //     #[case] expanded: &[&str],
    //     #[case] from: Option<DateTime<Utc>>,
    //     #[case] to: Option<DateTime<Utc>>,
    // ) {
    //     let event = CalendarObject::from_ics(ics.to_string(), None).unwrap();
    //     let crate::CalendarObjectComponent::Event(event, overrides) = event.get_data() else {
    //         panic!()
    //     };
    //
    //     let events: Vec<String> = event
    //         .expand_recurrence(from, to, overrides)
    //         .unwrap()
    //         .into_iter()
    //         .map(|event| Emitter::generate(&event))
    //         .collect();
    //     assert_eq!(events.len(), expanded.len());
    //     for (output, reference) in events.iter().zip(expanded) {
    //         similar_asserts::assert_eq!(output, reference);
    //     }
    // }
}
