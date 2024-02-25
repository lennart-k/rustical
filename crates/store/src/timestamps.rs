use lazy_static::lazy_static;
lazy_static! {
    static ref RE_DURATION: regex::Regex = regex::Regex::new(r"^(?<sign>[+-])?P((?P<W>\d+)W)?((?P<D>\d+)D)?(T((?P<H>\d+)H)?((?P<M>\d+)M)?((?P<S>\d+)S)?)?$").unwrap();
}
use anyhow::{anyhow, Result};
use chrono::{Duration, NaiveDateTime};

pub fn parse_duration(string: &str) -> Result<Duration> {
    let captures = RE_DURATION
        .captures(string)
        .ok_or(anyhow!("invalid duration format"))?;

    let mut duration = Duration::zero();
    if let Some(weeks) = captures.name("W") {
        duration = duration + Duration::weeks(weeks.as_str().parse()?);
    }
    if let Some(days) = captures.name("D") {
        duration = duration + Duration::days(days.as_str().parse()?);
    }
    if let Some(hours) = captures.name("H") {
        duration = duration + Duration::hours(hours.as_str().parse()?);
    }
    if let Some(minutes) = captures.name("M") {
        duration = duration + Duration::minutes(minutes.as_str().parse()?);
    }
    if let Some(seconds) = captures.name("S") {
        duration = duration + Duration::seconds(seconds.as_str().parse()?);
    }
    if let Some(sign) = captures.name("sign") {
        if sign.as_str() == "-" {
            duration = -duration;
        }
    }

    Ok(duration)
}

#[test]
fn test_parse_duration() {
    assert_eq!(parse_duration("P12W").unwrap(), Duration::weeks(12));
    assert_eq!(parse_duration("P12D").unwrap(), Duration::days(12));
    assert_eq!(parse_duration("PT12H").unwrap(), Duration::hours(12));
    assert_eq!(parse_duration("PT12M").unwrap(), Duration::minutes(12));
    assert_eq!(parse_duration("PT12S").unwrap(), Duration::seconds(12));
}

pub fn parse_datetime(string: &str) -> Result<NaiveDateTime> {
    // TODO: respect timezones
    //
    // Format: ^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})(?P<utc>Z)?$
    // if Z?
    //   UTC time
    // else
    //   if TZID given?
    //      time in TZ
    //   else
    //      local time of attendee (can be different actual times for different attendees)
    //      BUT for this implementation will be UTC for now since this case is annoying
    //      (sabre-dav does same)
    let (datetime, _tz_remainder) = NaiveDateTime::parse_and_remainder(string, "%Y%m%dT%H%M%S")?;
    Ok(datetime)
}

#[test]
fn test_parse_datetime() {
    dbg!(parse_datetime("19960329T133000Z").unwrap());
}
