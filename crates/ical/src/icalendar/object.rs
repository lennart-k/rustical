use crate::CalendarObjectType;
use crate::Error;
use ical::component::IcalCalendarObject;
use ical::parser::Component;
use ical::property::Property;
use sha2::{Digest, Sha256};
use std::io::BufReader;

#[derive(Debug, Clone)]
pub struct CalendarObject {
    id: String,
    ics: String,
    inner: IcalCalendarObject,
}

impl CalendarObject {
    pub fn from_ics(ics: String, id: Option<String>) -> Result<Self, Error> {
        let mut parser = ical::IcalObjectParser::new(BufReader::new(ics.as_bytes()));
        let cal = parser.next().ok_or(Error::MissingCalendar)??;
        if parser.next().is_some() {
            return Err(Error::InvalidData(
                "multiple calendars, only one allowed".to_owned(),
            ));
        }

        Ok(Self {
            id: id.unwrap_or_else(|| cal.get_uid().to_owned()),
            ics,
            inner: cal,
        })
    }

    #[must_use]
    pub const fn get_inner(&self) -> &IcalCalendarObject {
        &self.inner
    }

    #[must_use]
    pub fn get_id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.inner.get_uid());
        hasher.update(self.get_ics());
        format!("\"{:x}\"", hasher.finalize())
    }

    #[must_use]
    pub fn get_ics(&self) -> &str {
        &self.ics
    }

    #[must_use]
    pub fn get_object_type(&self) -> CalendarObjectType {
        (&self.inner).into()
    }

    #[must_use]
    pub fn get_property(&self, name: &str) -> Option<&Property> {
        self.inner.get_property(name)
    }
}
