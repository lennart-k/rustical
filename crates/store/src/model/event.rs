use crate::{
    timestamp::{parse_duration, CalDateTime},
    Error,
};
use anyhow::{anyhow, Result};
use chrono::Duration;
use ical::{
    generator::IcalEvent,
    parser::{ical::component::IcalTimeZone, Component},
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
        // This is safe since we enforce the event's existance in the constructor
        if let Some(dtstart) = self.event.get_property("DTSTART") {
            CalDateTime::parse_prop(dtstart, &self.timezones)
        } else {
            Ok(None)
        }
    }

    pub fn get_last_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        // This is safe since we enforce the event's existence in the constructor
        if self.event.get_property("RRULE").is_some() {
            // TODO: understand recurrence rules
            return Err(anyhow!("event is recurring, we cannot handle that yet").into());
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
}
