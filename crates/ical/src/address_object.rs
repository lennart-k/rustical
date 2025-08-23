use crate::{CalDateTime, LOCAL_DATE};
use crate::{CalendarObject, Error};
use chrono::Datelike;
use ical::generator::Emitter;
use ical::parser::{
    Component,
    vcard::{self, component::VcardContact},
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, io::BufReader};

#[derive(Debug, Clone)]
pub struct AddressObject {
    vcf: String,
    vcard: VcardContact,
}

impl TryFrom<VcardContact> for AddressObject {
    type Error = Error;

    fn try_from(vcard: VcardContact) -> Result<Self, Self::Error> {
        if vcard.get_uid().is_none() {
            return Err(Error::InvalidData("missing UID".to_owned()));
        }
        let vcf = vcard.generate();
        Ok(Self { vcf, vcard })
    }
}

impl AddressObject {
    pub fn from_vcf(vcf: String) -> Result<Self, Error> {
        let mut parser = vcard::VcardParser::new(BufReader::new(vcf.as_bytes()));
        let vcard = parser.next().ok_or(Error::MissingContact)??;
        if parser.next().is_some() {
            return Err(Error::InvalidData(
                "multiple vcards, only one allowed".to_owned(),
            ));
        }

        if vcard.get_uid().is_none() {
            return Err(Error::InvalidData("missing UID".to_owned()));
        }
        Ok(Self { vcf, vcard })
    }

    pub fn get_id(&self) -> &str {
        self.vcard
            .get_uid()
            .expect("We've validated before that UID exists")
    }

    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.get_id());
        hasher.update(self.get_vcf());
        format!("\"{:x}\"", hasher.finalize())
    }

    pub fn get_vcf(&self) -> &str {
        &self.vcf
    }

    pub fn get_anniversary(&self) -> Option<(CalDateTime, bool)> {
        let prop = self.vcard.get_property("ANNIVERSARY")?.value.as_deref()?;
        CalDateTime::parse_vcard(prop).ok()
    }

    pub fn get_birthday(&self) -> Option<(CalDateTime, bool)> {
        let prop = self.vcard.get_property("BDAY")?.value.as_deref()?;
        CalDateTime::parse_vcard(prop).ok()
    }

    pub fn get_full_name(&self) -> Option<&str> {
        let prop = self.vcard.get_property("FN")?;
        prop.value.as_deref()
    }

    pub fn get_anniversary_object(&self) -> Result<Option<CalendarObject>, Error> {
        Ok(
            if let Some((anniversary, contains_year)) = self.get_anniversary() {
                let fullname = if let Some(name) = self.get_full_name() {
                    name
                } else {
                    return Ok(None);
                };
                let anniversary = anniversary.date();
                let year = contains_year.then_some(anniversary.year());
                let anniversary_start = anniversary.format(LOCAL_DATE);
                let anniversary_end = anniversary
                    .succ_opt()
                    .unwrap_or(anniversary)
                    .format(LOCAL_DATE);
                let uid = format!("{}-anniversary", self.get_id());

                let year_suffix = year.map(|year| format!(" ({year})")).unwrap_or_default();
                Some(CalendarObject::from_ics(format!(
                    r#"BEGIN:VCALENDAR
VERSION:2.0
CALSCALE:GREGORIAN
PRODID:-//github.com/lennart-k/rustical birthday calendar//EN
BEGIN:VEVENT
DTSTART;VALUE=DATE:{anniversary_start}
DTEND;VALUE=DATE:{anniversary_end}
UID:{uid}
RRULE:FREQ=YEARLY
SUMMARY:💍 {fullname}{year_suffix}
TRANSP:TRANSPARENT
BEGIN:VALARM
TRIGGER;VALUE=DURATION:-PT0M
ACTION:DISPLAY
DESCRIPTION:💍 {fullname}{year_suffix}
END:VALARM
END:VEVENT
END:VCALENDAR"#,
                ))?)
            } else {
                None
            },
        )
    }

    pub fn get_birthday_object(&self) -> Result<Option<CalendarObject>, Error> {
        Ok(
            if let Some((birthday, contains_year)) = self.get_birthday() {
                let fullname = if let Some(name) = self.get_full_name() {
                    name
                } else {
                    return Ok(None);
                };
                let birthday = birthday.date();
                let year = contains_year.then_some(birthday.year());
                let birthday_start = birthday.format(LOCAL_DATE);
                let birthday_end = birthday.succ_opt().unwrap_or(birthday).format(LOCAL_DATE);
                let uid = format!("{}-birthday", self.get_id());

                let year_suffix = year.map(|year| format!(" ({year})")).unwrap_or_default();
                Some(CalendarObject::from_ics(format!(
                    r#"BEGIN:VCALENDAR
VERSION:2.0
CALSCALE:GREGORIAN
PRODID:-//github.com/lennart-k/rustical birthday calendar//EN
BEGIN:VEVENT
DTSTART;VALUE=DATE:{birthday_start}
DTEND;VALUE=DATE:{birthday_end}
UID:{uid}
RRULE:FREQ=YEARLY
SUMMARY:🎂 {fullname}{year_suffix}
TRANSP:TRANSPARENT
BEGIN:VALARM
TRIGGER;VALUE=DURATION:-PT0M
ACTION:DISPLAY
DESCRIPTION:🎂 {fullname}{year_suffix}
END:VALARM
END:VEVENT
END:VCALENDAR"#,
                ))?)
            } else {
                None
            },
        )
    }

    /// Get significant dates associated with this address object
    pub fn get_significant_dates(&self) -> Result<HashMap<&'static str, CalendarObject>, Error> {
        let mut out = HashMap::new();
        if let Some(birthday) = self.get_birthday_object()? {
            out.insert("birthday", birthday);
        }
        if let Some(anniversary) = self.get_anniversary_object()? {
            out.insert("anniversary", anniversary);
        }
        Ok(out)
    }
}
