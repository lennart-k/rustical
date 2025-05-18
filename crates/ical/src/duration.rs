use crate::CalDateTimeError;
use chrono::Duration;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_DURATION: regex::Regex = regex::Regex::new(r"^(?<sign>[+-])?P((?P<W>\d+)W)?((?P<D>\d+)D)?(T((?P<H>\d+)H)?((?P<M>\d+)M)?((?P<S>\d+)S)?)?$").unwrap();
}

pub fn parse_duration(string: &str) -> Result<Duration, CalDateTimeError> {
    let captures = RE_DURATION
        .captures(string)
        .ok_or(CalDateTimeError::InvalidDurationFormat(string.to_string()))?;

    let mut duration = Duration::zero();
    if let Some(weeks) = captures.name("W") {
        duration += Duration::weeks(weeks.as_str().parse().unwrap());
    }
    if let Some(days) = captures.name("D") {
        duration += Duration::days(days.as_str().parse().unwrap());
    }
    if let Some(hours) = captures.name("H") {
        duration += Duration::hours(hours.as_str().parse().unwrap());
    }
    if let Some(minutes) = captures.name("M") {
        duration += Duration::minutes(minutes.as_str().parse().unwrap());
    }
    if let Some(seconds) = captures.name("S") {
        duration += Duration::seconds(seconds.as_str().parse().unwrap());
    }
    if let Some(sign) = captures.name("sign") {
        if sign.as_str() == "-" {
            duration = -duration;
        }
    }

    Ok(duration)
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::parse_duration;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("P12W").unwrap(), Duration::weeks(12));
        assert_eq!(parse_duration("P12D").unwrap(), Duration::days(12));
        assert_eq!(parse_duration("PT12H").unwrap(), Duration::hours(12));
        assert_eq!(parse_duration("PT12M").unwrap(), Duration::minutes(12));
        assert_eq!(parse_duration("PT12S").unwrap(), Duration::seconds(12));
    }
}
