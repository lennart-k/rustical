use crate::{CalendarObject, Error};
use caldata::{
    IcalAlarmBuilder, IcalCalendarObjectBuilder, IcalEventBuilder, VcardParser,
    component::{ComponentMut, vcard::VcardContact},
    generator::Emitter,
    parser::{ContentLine, ParserOptions},
    property::{
        Calscale, IcalCALSCALEProperty, IcalDTENDProperty, IcalDTSTAMPProperty,
        IcalDTSTARTProperty, IcalPRODIDProperty, IcalRRULEProperty, IcalSUMMARYProperty,
        IcalUIDProperty, IcalVERSIONProperty, IcalVersion, VcardANNIVERSARYProperty,
        VcardBDAYProperty, VcardFNProperty,
    },
    types::{CalDate, CalDateOrDateTime, PartialDate},
};
use chrono::{NaiveDate, Utc};
use hex::ToHex;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Debug, Clone)]
pub struct AddressObject {
    inner: VcardContact,
    vcf: OnceLock<String>,
}

impl From<VcardContact> for AddressObject {
    fn from(vcard: VcardContact) -> Self {
        Self {
            inner: vcard,
            vcf: OnceLock::new(),
        }
    }
}

impl AddressObject {
    pub fn from_vcf(vcf: String) -> Result<Self, Error> {
        let parser = VcardParser::from_slice(vcf.as_bytes());
        let vcard = parser.expect_one()?;
        Ok(Self {
            inner: vcard,
            vcf: vcf.into(),
        })
    }

    #[must_use]
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.get_vcf());
        format!(
            "\"{}\"",
            hasher.finalize().as_slice().encode_hex::<String>()
        )
    }

    #[must_use]
    pub fn get_vcf(&self) -> &str {
        self.vcf.get_or_init(|| self.inner.generate_string())
    }

    fn get_significant_date_object(
        &self,
        date: &PartialDate,
        summary_prefix: &str,
        suffix: &str,
    ) -> Result<Option<CalendarObject>, Error> {
        let Some(uid) = self.inner.get_uid() else {
            return Ok(None);
        };
        let uid = format!("{uid}{suffix}");
        let year = date.get_year();
        let year_suffix = year.map_or_default(|year| format!(" {year}"));
        let Some(month) = date.get_month() else {
            return Ok(None);
        };
        let Some(day) = date.get_day() else {
            return Ok(None);
        };
        let Some(dtstart) = NaiveDate::from_ymd_opt(year.unwrap_or(1900), month, day) else {
            return Ok(None);
        };
        let Some(end_date) = dtstart.succ_opt() else {
            // start_date is MAX_DATE, this should never happen but FAPP also not raise an error
            return Ok(None);
        };
        let Some(VcardFNProperty(fullname, _)) = self.inner.full_name.first() else {
            return Ok(None);
        };
        let summary = format!("{summary_prefix} {fullname}{year_suffix}");

        let event = IcalEventBuilder {
            content_lines: vec![
                IcalDTSTAMPProperty(Utc::now().into(), vec![].into()).into(),
                IcalDTSTARTProperty(CalDateOrDateTime::Date(CalDate(dtstart)), vec![].into())
                    .into(),
                IcalDTENDProperty(CalDateOrDateTime::Date(CalDate(end_date)), vec![].into()).into(),
                IcalUIDProperty(uid, vec![].into()).into(),
                IcalRRULEProperty(
                    caldata::rrule::RRule::from_str("FREQ=YEARLY").unwrap(),
                    vec![].into(),
                )
                .into(),
                IcalSUMMARYProperty(summary.clone(), vec![].into()).into(),
                ContentLine {
                    name: "TRANSP".to_owned(),
                    value: "TRANSPARENT".to_owned(),
                    ..Default::default()
                },
            ],
            alarms: vec![IcalAlarmBuilder {
                content_lines: vec![
                    ContentLine {
                        name: "TRIGGER".to_owned(),
                        value: "-PT0M".to_owned(),
                        params: vec![("VALUE".to_owned(), vec!["DURATION".to_owned()])].into(),
                    },
                    ContentLine {
                        name: "ACTION".to_owned(),
                        value: "DISPLAY".to_owned(),
                        ..Default::default()
                    },
                    ContentLine {
                        name: "DESCRIPTION".to_owned(),
                        value: summary,
                        ..Default::default()
                    },
                ],
            }],
        };

        Ok(Some(
            IcalCalendarObjectBuilder {
                content_lines: vec![
                    IcalVERSIONProperty(IcalVersion::Version2_0, vec![].into()).into(),
                    IcalCALSCALEProperty(Calscale::Gregorian, vec![].into()).into(),
                    IcalPRODIDProperty(
                        "-//github.com/lennart-k/rustical birthday calendar//EN".to_owned(),
                        vec![].into(),
                    )
                    .into(),
                ],
                inner: Some(caldata::CalendarInnerDataBuilder::Event(vec![event])),
                vtimezones: BTreeMap::default(),
            }
            .build(&ParserOptions::default(), None)?
            .into(),
        ))
    }

    pub fn get_anniversary_object(&self) -> Result<Option<CalendarObject>, Error> {
        let Some(VcardANNIVERSARYProperty(anniversary, _)) = &self.inner.anniversary else {
            return Ok(None);
        };
        let Some(date) = &anniversary.date else {
            return Ok(None);
        };

        self.get_significant_date_object(date, "💍", "-anniversary")
    }

    pub fn get_birthday_object(&self) -> Result<Option<CalendarObject>, Error> {
        let Some(VcardBDAYProperty(bday, _)) = &self.inner.bday else {
            return Ok(None);
        };
        let Some(date) = &bday.date else {
            return Ok(None);
        };

        self.get_significant_date_object(date, "🎂", "-birthday")
    }

    #[must_use]
    pub const fn get_vcard(&self) -> &VcardContact {
        &self.inner
    }
}

#[cfg(any(test, feature = "test"))]
impl AddressObject {
    #[must_use]
    pub fn example_minimal() -> Self {
        Self::from_vcf(
            "BEGIN:VCARD
VERSION:4.0
FN:Jon Doe
N:Doe;Jon;;;
END:VCARD"
                .to_string(),
        )
        .unwrap()
    }
}
