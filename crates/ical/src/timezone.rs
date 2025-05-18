use chrono::{Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use derive_more::{Display, From};

#[derive(Debug, Clone, From)]
pub enum CalTimezone {
    Local,
    Utc,
    Olson(Tz),
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum CalTimezoneOffset {
    Local(chrono::FixedOffset),
    Utc(chrono::Utc),
    Olson(chrono_tz::TzOffset),
}

impl chrono::Offset for CalTimezoneOffset {
    fn fix(&self) -> chrono::FixedOffset {
        match self {
            Self::Local(local) => local.fix(),
            Self::Utc(utc) => utc.fix(),
            Self::Olson(olson) => olson.fix(),
        }
    }
}

impl TimeZone for CalTimezone {
    type Offset = CalTimezoneOffset;

    fn from_offset(offset: &Self::Offset) -> Self {
        match offset {
            CalTimezoneOffset::Local(_) => Self::Local,
            CalTimezoneOffset::Utc(_) => Self::Utc,
            CalTimezoneOffset::Olson(offset) => Self::Olson(Tz::from_offset(offset)),
        }
    }

    fn offset_from_local_date(&self, local: &NaiveDate) -> chrono::MappedLocalTime<Self::Offset> {
        match self {
            Self::Local => Local
                .offset_from_local_date(local)
                .map(CalTimezoneOffset::Local),
            Self::Utc => Utc
                .offset_from_local_date(local)
                .map(CalTimezoneOffset::Utc),
            Self::Olson(tz) => tz
                .offset_from_local_date(local)
                .map(CalTimezoneOffset::Olson),
        }
    }

    fn offset_from_local_datetime(
        &self,
        local: &NaiveDateTime,
    ) -> chrono::MappedLocalTime<Self::Offset> {
        match self {
            Self::Local => Local
                .offset_from_local_datetime(local)
                .map(CalTimezoneOffset::Local),
            Self::Utc => Utc
                .offset_from_local_datetime(local)
                .map(CalTimezoneOffset::Utc),
            Self::Olson(tz) => tz
                .offset_from_local_datetime(local)
                .map(CalTimezoneOffset::Olson),
        }
    }

    fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> Self::Offset {
        match self {
            Self::Local => CalTimezoneOffset::Local(Local.offset_from_utc_datetime(utc)),
            Self::Utc => CalTimezoneOffset::Utc(Utc.offset_from_utc_datetime(utc)),
            Self::Olson(tz) => CalTimezoneOffset::Olson(tz.offset_from_utc_datetime(utc)),
        }
    }

    fn offset_from_utc_date(&self, utc: &NaiveDate) -> Self::Offset {
        match self {
            Self::Local => CalTimezoneOffset::Local(Local.offset_from_utc_date(utc)),
            Self::Utc => CalTimezoneOffset::Utc(Utc.offset_from_utc_date(utc)),
            Self::Olson(tz) => CalTimezoneOffset::Olson(tz.offset_from_utc_date(utc)),
        }
    }
}
