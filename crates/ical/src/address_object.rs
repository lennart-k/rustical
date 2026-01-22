use crate::{CalendarObject, Error};
use caldata::{
    VcardParser,
    component::{
        CalendarInnerDataBuilder, ComponentMut, IcalAlarmBuilder, IcalCalendarObjectBuilder,
        IcalEventBuilder, VcardContact,
    },
    generator::Emitter,
    parser::ContentLine,
    property::{
        Calscale, IcalCALSCALEProperty, IcalDTENDProperty, IcalDTSTAMPProperty,
        IcalDTSTARTProperty, IcalPRODIDProperty, IcalRRULEProperty, IcalSUMMARYProperty,
        IcalUIDProperty, IcalVERSIONProperty, IcalVersion, VcardANNIVERSARYProperty,
        VcardBDAYProperty, VcardFNProperty,
    },
    types::{CalDate, PartialDate, Timezone},
};
use chrono::{NaiveDate, Utc};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct AddressObject {
    vcf: String,
    vcard: VcardContact,
}

impl From<VcardContact> for AddressObject {
    fn from(vcard: VcardContact) -> Self {
        let vcf = vcard.generate();
        Self { vcf, vcard }
    }
}

impl AddressObject {
    pub fn from_vcf(vcf: String) -> Result<Self, Error> {
        let parser = VcardParser::from_slice(vcf.as_bytes());
        let vcard = parser.expect_one()?;
        Ok(Self { vcf, vcard })
    }

    #[must_use]
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.get_vcf());
        format!("\"{:x}\"", hasher.finalize())
    }

    #[must_use]
    pub fn get_vcf(&self) -> &str {
        &self.vcf
    }

    fn get_significant_date_object(
        &self,
        date: &PartialDate,
        summary_prefix: &str,
        suffix: &str,
    ) -> Result<Option<CalendarObject>, Error> {
        let Some(uid) = self.vcard.get_uid() else {
            return Ok(None);
        };
        let uid = format!("{uid}{suffix}");
        let year = date.get_year();
        let year_suffix = year.map(|year| format!(" {year}")).unwrap_or_default();
        let Some(month) = date.get_month() else {
            return Ok(None);
        };
        let Some(day) = date.get_day() else {
            return Ok(None);
        };
        let Some(dtstart) = NaiveDate::from_ymd_opt(year.unwrap_or(1900), month, day) else {
            return Ok(None);
        };
        let start_date = CalDate(dtstart, Timezone::Local);
        let Some(end_date) = start_date.succ_opt() else {
            // start_date is MAX_DATE, this should never happen but FAPP also not raise an error
            return Ok(None);
        };
        let Some(VcardFNProperty(fullname, _)) = self.vcard.full_name.first() else {
            return Ok(None);
        };
        let summary = format!("{summary_prefix} {fullname}{year_suffix}");

        let event = IcalEventBuilder {
            properties: vec![
                IcalDTSTAMPProperty(Utc::now().into(), vec![].into()).into(),
                IcalDTSTARTProperty(start_date.into(), vec![].into()).into(),
                IcalDTENDProperty(end_date.into(), vec![].into()).into(),
                IcalUIDProperty(uid, vec![].into()).into(),
                IcalRRULEProperty(
                    rrule::RRule::from_str("FREQ=YEARLY").unwrap(),
                    vec![].into(),
                )
                .into(),
                IcalSUMMARYProperty(summary.clone(), vec![].into()).into(),
                ContentLine {
                    name: "TRANSP".to_owned(),
                    value: Some("TRANSPARENT".to_owned()),
                    ..Default::default()
                },
            ],
            alarms: vec![IcalAlarmBuilder {
                properties: vec![
                    ContentLine {
                        name: "TRIGGER".to_owned(),
                        value: Some("-PT0M".to_owned()),
                        params: vec![("VALUE".to_owned(), vec!["DURATION".to_owned()])].into(),
                    },
                    ContentLine {
                        name: "ACTION".to_owned(),
                        value: Some("DISPLAY".to_owned()),
                        ..Default::default()
                    },
                    ContentLine {
                        name: "DESCRIPTION".to_owned(),
                        value: Some(summary),
                        ..Default::default()
                    },
                ],
            }],
        };

        Ok(Some(
            IcalCalendarObjectBuilder {
                properties: vec![
                    IcalVERSIONProperty(IcalVersion::Version2_0, vec![].into()).into(),
                    IcalCALSCALEProperty(Calscale::Gregorian, vec![].into()).into(),
                    IcalPRODIDProperty(
                        "-//github.com/lennart-k/rustical birthday calendar//EN".to_owned(),
                        vec![].into(),
                    )
                    .into(),
                ],
                inner: Some(CalendarInnerDataBuilder::Event(vec![event])),
                vtimezones: HashMap::default(),
            }
            .build(None)?
            .into(),
        ))
    }

    pub fn get_anniversary_object(&self) -> Result<Option<CalendarObject>, Error> {
        let Some(VcardANNIVERSARYProperty(anniversary, _)) = &self.vcard.anniversary else {
            return Ok(None);
        };
        let Some(date) = &anniversary.date else {
            return Ok(None);
        };

        self.get_significant_date_object(date, "💍", "-anniversary")
    }

    pub fn get_birthday_object(&self) -> Result<Option<CalendarObject>, Error> {
        let Some(VcardBDAYProperty(bday, _)) = &self.vcard.birthday else {
            return Ok(None);
        };
        let Some(date) = &bday.date else {
            return Ok(None);
        };

        self.get_significant_date_object(date, "🎂", "-birthday")
    }

    #[must_use]
    pub const fn get_vcard(&self) -> &VcardContact {
        &self.vcard
    }
}
