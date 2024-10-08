use std::str::FromStr;

use anyhow::{anyhow, Result};
use chrono::Duration;
use ical::{generator::IcalEvent, parser::Component, property::Property};

use crate::{
    timestamp::{parse_duration, CalDateTime},
    Error,
};

#[derive(Debug, Clone)]
pub struct EventObject {
    pub(crate) event: IcalEvent,
}

impl EventObject {
    pub fn get_first_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        // This is safe since we enforce the event's existance in the constructor
        let dtstart = if let Some(Property {
            value: Some(value), ..
        }) = self.event.get_property("DTSTART")
        {
            value
        } else {
            return Ok(None);
        };
        Ok(Some(CalDateTime::from_str(&dtstart)?))
    }

    pub fn get_last_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        // This is safe since we enforce the event's existence in the constructor
        if self.event.get_property("RRULE").is_some() {
            // TODO: understand recurrence rules
            return Err(anyhow!("event is recurring, we cannot handle that yet").into());
        }

        if let Some(Property {
            value: Some(dtend), ..
        }) = self.event.get_property("DTEND")
        {
            return Ok(Some(CalDateTime::from_str(&dtend)?));
        };

        let duration = if let Some(Property {
            value: Some(duration),
            ..
        }) = self.event.get_property("DURATION")
        {
            parse_duration(&duration)?
        } else {
            Duration::days(1)
        };

        let first_occurence = self.get_first_occurence()?;
        return Ok(first_occurence.map(|first_occurence| first_occurence + duration));
    }
}
