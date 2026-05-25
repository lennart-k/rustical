use crate::{CalendarObject, Error};
use caldata::{
    VcardParser,
    component::{
        CalendarInnerDataBuilder, ComponentMut, IcalAlarmBuilder, IcalCalendarObjectBuilder,
        IcalEventBuilder, VcardContact,
    },
    generator::Emitter,
    parser::{ContentLine, ParserOptions},
    property::{
        Calscale, IcalCALSCALEProperty, IcalDTENDProperty, IcalDTSTAMPProperty,
        IcalDTSTARTProperty, IcalPRODIDProperty, IcalRECURIDProperty, IcalRRULEProperty,
        IcalSUMMARYProperty, IcalUIDProperty, IcalVERSIONProperty, IcalVersion, RecurIdRange,
        VcardANNIVERSARYProperty, VcardBDAYProperty, VcardFNProperty,
    },
    types::{CalDate, PartialDate, Tz},
};
use chrono::{Datelike, NaiveDate, Utc};
use hex::ToHex;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
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
        format!(
            "\"{}\"",
            hasher.finalize().as_slice().encode_hex::<String>()
        )
    }

    #[must_use]
    pub fn get_vcf(&self) -> &str {
        &self.vcf
    }

    fn get_significant_date_object(
        &self,
        date: &PartialDate,
        this_year: i32,
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
        let start_date = CalDate(dtstart, Tz::Local);
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
                IcalDTSTARTProperty(start_date.clone().into(), vec![].into()).into(),
                IcalDTENDProperty(end_date.clone().into(), vec![].into()).into(),
                IcalUIDProperty(uid.clone(), vec![].into()).into(),
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
                properties: vec![
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

        let mut events = vec![event];
        if let Some(y) = year {
            if let Some(dtstart) = NaiveDate::from_ymd_opt(y, month, day) {
                if let Some(this_year_instance) = create_recurring_instance(
                    uid.clone(),
                    summary_prefix,
                    fullname,
                    dtstart,
                    this_year,
                ) {
                    events.push(this_year_instance);
                }
                if let Some(next_year_instance) =
                    create_recurring_instance(uid, summary_prefix, fullname, dtstart, this_year + 1)
                {
                    events.push(next_year_instance);
                }
            }
        }

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
                inner: Some(CalendarInnerDataBuilder::Event(events)),
                vtimezones: BTreeMap::default(),
            }
            .build(&ParserOptions::default(), None)?
            .into(),
        ))
    }
    pub fn get_anniversary_object(&self, year: i32) -> Result<Option<CalendarObject>, Error> {
        let Some(VcardANNIVERSARYProperty(anniversary, _)) = &self.vcard.anniversary else {
            return Ok(None);
        };
        let Some(date) = &anniversary.date else {
            return Ok(None);
        };

        self.get_significant_date_object(date, year, "💍", "-anniversary")
    }

    pub fn get_birthday_object(&self, year: i32) -> Result<Option<CalendarObject>, Error> {
        let Some(VcardBDAYProperty(bday, _)) = &self.vcard.birthday else {
            return Ok(None);
        };
        let Some(date) = &bday.date else {
            return Ok(None);
        };

        self.get_significant_date_object(date, year, "🎂", "-birthday")
    }

    #[must_use]
    pub const fn get_vcard(&self) -> &VcardContact {
        &self.vcard
    }
}

fn create_recurring_instance(
    uid: String,
    summary_prefix: &str,
    fullname: &str,
    dtstart: NaiveDate,
    year: i32,
) -> Option<IcalEventBuilder> {
    let Some(dt_this_year) = NaiveDate::from_ymd_opt(year, dtstart.month(), dtstart.day()) else {
        return None;
    };
    let this_year_start_date = CalDate(dt_this_year, Tz::Local);
    let Some(this_year_end_date) = this_year_start_date.succ_opt() else {
        // this_year_start_date is MAX_DATE, this should never happen but FAPP also not raise an error
        return None;
    };
    let age_suffix = dt_this_year
        .years_since(dtstart)
        .map(|age| format!(" {age}"))
        .unwrap_or_default();
    let this_year_summary = format!("{summary_prefix} {fullname}{age_suffix}");
    return Some(IcalEventBuilder {
        properties: vec![
            IcalDTSTAMPProperty(Utc::now().into(), vec![].into()).into(),
            IcalDTSTARTProperty(this_year_start_date.clone().into(), vec![].into()).into(),
            IcalDTENDProperty(this_year_end_date.clone().into(), vec![].into()).into(),
            IcalUIDProperty(uid, vec![].into()).into(),
            IcalRECURIDProperty(
                this_year_start_date.into(),
                vec![].into(),
                RecurIdRange::This,
            )
            .into(),
            IcalSUMMARYProperty(this_year_summary.clone(), vec![].into()).into(),
            ContentLine {
                name: "TRANSP".to_owned(),
                value: "TRANSPARENT".to_owned(),
                ..Default::default()
            },
        ],
        alarms: vec![IcalAlarmBuilder {
            properties: vec![
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
                    value: this_year_summary,
                    ..Default::default()
                },
            ],
        }],
    });
}
