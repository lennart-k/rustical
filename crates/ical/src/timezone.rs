use chrono::{Local, NaiveDate, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use derive_more::{Display, From};

#[derive(Debug, Clone, From, PartialEq, Eq)]
pub enum ICalTimezone {
    Local,
    Olson(Tz),
}

impl From<ICalTimezone> for rrule::Tz {
    fn from(value: ICalTimezone) -> Self {
        match value {
            ICalTimezone::Local => Self::LOCAL,
            ICalTimezone::Olson(tz) => Self::Tz(tz),
        }
    }
}

impl From<rrule::Tz> for ICalTimezone {
    fn from(value: rrule::Tz) -> Self {
        match value {
            rrule::Tz::Local(_) => Self::Local,
            rrule::Tz::Tz(tz) => Self::Olson(tz),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum CalTimezoneOffset {
    Local(chrono::FixedOffset),
    Olson(chrono_tz::TzOffset),
}

impl chrono::Offset for CalTimezoneOffset {
    fn fix(&self) -> chrono::FixedOffset {
        match self {
            Self::Local(local) => local.fix(),
            Self::Olson(olson) => olson.fix(),
        }
    }
}

impl TimeZone for ICalTimezone {
    type Offset = CalTimezoneOffset;

    fn from_offset(offset: &Self::Offset) -> Self {
        match offset {
            CalTimezoneOffset::Local(_) => Self::Local,
            CalTimezoneOffset::Olson(offset) => Self::Olson(Tz::from_offset(offset)),
        }
    }

    fn offset_from_local_date(&self, local: &NaiveDate) -> chrono::MappedLocalTime<Self::Offset> {
        match self {
            Self::Local => Local
                .offset_from_local_date(local)
                .map(CalTimezoneOffset::Local),
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
            Self::Olson(tz) => tz
                .offset_from_local_datetime(local)
                .map(CalTimezoneOffset::Olson),
        }
    }

    fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> Self::Offset {
        match self {
            Self::Local => CalTimezoneOffset::Local(Local.offset_from_utc_datetime(utc)),
            Self::Olson(tz) => CalTimezoneOffset::Olson(tz.offset_from_utc_datetime(utc)),
        }
    }

    fn offset_from_utc_date(&self, utc: &NaiveDate) -> Self::Offset {
        match self {
            Self::Local => CalTimezoneOffset::Local(Local.offset_from_utc_date(utc)),
            Self::Olson(tz) => CalTimezoneOffset::Olson(tz.offset_from_utc_date(utc)),
        }
    }
}
