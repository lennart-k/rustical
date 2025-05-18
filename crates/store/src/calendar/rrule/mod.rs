use super::{CalDateTime, CalDateTimeError};
use std::{num::ParseIntError, str::FromStr};
use strum_macros::EnumString;

mod iter;

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Missing RRULE FREQ")]
    MissingFrequency,
    #[error("Invalid RRULE part: {0}")]
    InvalidPart(String),
    #[error(transparent)]
    StrumError(#[from] strum::ParseError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    CalDateTimeError(#[from] CalDateTimeError),
}

#[derive(Debug, Clone, EnumString, Default, PartialEq)]
#[strum(serialize_all = "UPPERCASE")]
pub enum RecurrenceFrequency {
    Secondly,
    Minutely,
    Hourly,
    #[default]
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, EnumString, PartialEq)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Weekday {
    Mo,
    Tu,
    We,
    Th,
    Fr,
    Sa,
    Su,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecurrenceLimit {
    Count(usize),
    Until(CalDateTime),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecurrenceRule {
    // Self-explanatory
    pub frequency: RecurrenceFrequency,
    pub limit: Option<RecurrenceLimit>,
    // Repeat every n-th time
    pub interval: Option<usize>,

    pub bysecond: Option<Vec<usize>>,
    pub byminute: Option<Vec<usize>>,
    pub byhour: Option<Vec<usize>>,
    pub byday: Option<Vec<(Option<i64>, Weekday)>>,
    pub bymonthday: Option<Vec<i8>>,
    pub byyearday: Option<Vec<i64>>,
    pub byweekno: Option<Vec<i8>>,
    pub bymonth: Option<Vec<i8>>,
    pub week_start: Option<Weekday>,
    // Selects the n-th occurence within an a recurrence rule
    pub bysetpos: Option<Vec<i64>>,
}

impl RecurrenceRule {
    pub fn parse(rule: &str) -> Result<Self, ParserError> {
        let mut frequency = None;
        let mut limit = None;
        let mut interval = None;
        let mut bysecond = None;
        let mut byminute = None;
        let mut byhour = None;
        let mut byday = None;
        let mut bymonthday = None;
        let mut byyearday = None;
        let mut byweekno = None;
        let mut bymonth = None;
        let mut week_start = None;
        let mut bysetpos = None;

        for part in rule.split(';') {
            match part
                .split_once('=')
                .ok_or(ParserError::InvalidPart(part.to_owned()))?
            {
                ("FREQ", val) => {
                    frequency = Some(RecurrenceFrequency::from_str(val)?);
                }
                ("COUNT", val) => limit = Some(RecurrenceLimit::Count(val.parse()?)),
                ("UNTIL", val) => {
                    limit = Some(RecurrenceLimit::Until(CalDateTime::parse(val, None)?))
                }
                ("INTERVAL", val) => interval = Some(val.parse()?),
                ("BYSECOND", val) => {
                    bysecond = Some(
                        val.split(',')
                            .map(|val| val.parse())
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                ("BYMINUTE", val) => {
                    byminute = Some(
                        val.split(',')
                            .map(|val| val.parse())
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                ("BYHOUR", val) => {
                    byhour = Some(
                        val.split(',')
                            .map(|val| val.parse())
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                ("BYDAY", val) => {
                    byday = Some(
                        val.split(',')
                            .map(|val| {
                                assert!(val.len() >= 2);
                                let weekday =
                                    Weekday::from_str(val.get((val.len() - 2)..).unwrap())?;
                                let prefix = if val.len() > 2 {
                                    Some(val.get(..(val.len() - 2)).unwrap().parse()?)
                                } else {
                                    None
                                };
                                Ok((prefix, weekday))
                            })
                            .collect::<Result<Vec<_>, ParserError>>()?,
                    );
                }
                ("BYMONTHDAY", val) => {
                    bymonthday = Some(
                        val.split(',')
                            .map(|val| val.parse())
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                ("BYYEARDAY", val) => {
                    byyearday = Some(
                        val.split(',')
                            .map(|val| val.parse())
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                ("BYWEEKNO", val) => {
                    byweekno = Some(
                        val.split(',')
                            .map(|val| val.parse())
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                ("BYMONTH", val) => {
                    bymonth = Some(
                        val.split(',')
                            .map(|val| val.parse())
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                ("WKST", val) => week_start = Some(Weekday::from_str(val)?),
                ("BYSETPOS", val) => {
                    bysetpos = Some(
                        val.split(',')
                            .map(|val| val.parse())
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                (name, val) => panic!("Cannot handle {name}={val}"),
            }
        }
        Ok(Self {
            frequency: frequency.ok_or(ParserError::MissingFrequency)?,
            limit,
            interval,
            bysecond,
            byminute,
            byhour,
            byday,
            bymonthday,
            byyearday,
            byweekno,
            bymonth,
            week_start,
            bysetpos,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::calendar::{
        CalDateTime,
        rrule::{RecurrenceFrequency, RecurrenceLimit, Weekday},
    };

    use super::{ParserError, RecurrenceRule};

    #[test]
    fn parse_recurrence_rule() -> Result<(), ParserError> {
        assert_eq!(
            RecurrenceRule::parse("FREQ=DAILY;UNTIL=20250516T133000Z;INTERVAL=3")?,
            RecurrenceRule {
                frequency: RecurrenceFrequency::Daily,
                limit: Some(RecurrenceLimit::Until(
                    CalDateTime::parse("20250516T133000Z", None).unwrap()
                )),
                interval: Some(3),
                ..Default::default()
            }
        );
        assert_eq!(
            RecurrenceRule::parse("FREQ=WEEKLY;COUNT=4;INTERVAL=2;BYDAY=TU,TH,SU")?,
            RecurrenceRule {
                frequency: RecurrenceFrequency::Weekly,
                limit: Some(RecurrenceLimit::Count(4)),
                interval: Some(2),
                byday: Some(vec![
                    (None, Weekday::Tu),
                    (None, Weekday::Th),
                    (None, Weekday::Su),
                ]),
                ..Default::default()
            }
        );
        // Example: Last workday of the month
        assert_eq!(
            RecurrenceRule::parse("FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1")?,
            RecurrenceRule {
                frequency: RecurrenceFrequency::Monthly,
                byday: Some(vec![
                    (None, Weekday::Mo),
                    (None, Weekday::Tu),
                    (None, Weekday::We),
                    (None, Weekday::Th),
                    (None, Weekday::Fr),
                ]),
                bysetpos: Some(vec![-1]),
                ..Default::default()
            }
        );

        // Every last Sunday of March
        assert_eq!(
            RecurrenceRule::parse("FREQ=YEARLY;UNTIL=20370329T010000Z;BYDAY=-1SU;BYMONTH=3")?,
            RecurrenceRule {
                frequency: RecurrenceFrequency::Yearly,
                limit: Some(RecurrenceLimit::Until(
                    CalDateTime::parse("20370329T010000Z", None).unwrap()
                )),
                byday: Some(vec![(Some(-1), Weekday::Su)]),
                bymonth: Some(vec![3]),
                ..Default::default()
            }
        );

        Ok(())
    }
}
