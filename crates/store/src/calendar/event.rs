use super::{
    CalDateTime, parse_duration,
    rrule::{ParserError, RecurrenceRule},
};
use crate::Error;
use chrono::Duration;
use ical::{
    generator::IcalEvent,
    parser::{Component, ical::component::IcalTimeZone},
    property::Property,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EventObject {
    pub(crate) event: IcalEvent,
    pub(crate) timezones: HashMap<String, IcalTimeZone>,
}

impl EventObject {
    pub fn get_first_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        if let Some(dtstart) = self.event.get_property("DTSTART") {
            CalDateTime::parse_prop(dtstart, &self.timezones)
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
            return CalDateTime::parse_prop(dtend, &self.timezones);
        };

        let duration = if let Some(Property {
            value: Some(duration),
            ..
        }) = self.event.get_property("DURATION")
        {
            parse_duration(duration)?
        } else {
            Duration::days(1)
        };

        let first_occurence = self.get_first_occurence()?;
        Ok(first_occurence.map(|first_occurence| first_occurence + duration))
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

    pub fn expand_recurrence(&self) -> Result<(), Error> {
        let rrule = self.event.get_property("RRULE").unwrap();
        dbg!(rrule);
        Ok(())
    }
}

