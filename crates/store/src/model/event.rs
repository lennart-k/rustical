use std::str::FromStr;

use anyhow::{anyhow, Result};
use chrono::Duration;
use ical::{generator::IcalEvent, parser::Component};

use crate::timestamp::{parse_duration, CalDateTime};

#[derive(Debug, Clone)]
pub struct EventObject {
    pub(crate) event: IcalEvent,
}

impl EventObject {
    pub fn get_first_occurence(&self) -> Result<CalDateTime> {
        // This is safe since we enforce the event's existance in the constructor
        let dtstart = self
            .event
            .get_property("DTSTART")
            .ok_or(anyhow!("DTSTART property missing!"))?
            .value
            .to_owned()
            .ok_or(anyhow!("DTSTART property has no value!"))?;
        Ok(CalDateTime::from_str(&dtstart)?)
    }

    pub fn get_last_occurence(&self) -> Result<CalDateTime> {
        // This is safe since we enforce the event's existence in the constructor
        if self.event.get_property("RRULE").is_some() {
            // TODO: understand recurrence rules
            return Err(anyhow!("event is recurring, we cannot handle that yet"));
        }

        if let Some(dtend_prop) = self.event.get_property("DTEND") {
            let dtend = dtend_prop
                .value
                .to_owned()
                .ok_or(anyhow!("DTEND property has no value!"))?;
            return Ok(CalDateTime::from_str(&dtend)?);
        }

        if let Some(dtend_prop) = self.event.get_property("DURATION") {
            let duration = dtend_prop
                .value
                .to_owned()
                .ok_or(anyhow!("DURATION property has no value!"))?;
            let dtstart = self.get_first_occurence()?;
            return Ok(dtstart + parse_duration(&duration)?);
        }

        let dtstart = self.get_first_occurence()?;
        if let CalDateTime::Date(_) = dtstart {
            return Ok(dtstart + Duration::days(1));
        }

        Err(anyhow!("help, couldn't determine any last occurence"))
    }
}
