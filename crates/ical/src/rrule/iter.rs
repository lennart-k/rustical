use crate::CalDateTime;

use super::{RecurrenceLimit, RecurrenceRule};

impl RecurrenceRule {
    pub fn between(
        &self,
        start: CalDateTime,
        end: Option<CalDateTime>,
        limit: Option<usize>,
    ) -> Vec<CalDateTime> {
        let start = start;
        // Terrible code, should clean this up later.
        let mut end = end;
        if let Some(RecurrenceLimit::Until(until)) = &self.limit {
            let mut _end = end.unwrap_or(until.clone());
            if until.utc() < _end.utc() {
                _end = until.clone();
            }
            end = Some(_end);
        }
        let mut count = if let Some(RecurrenceLimit::Count(count)) = &self.limit {
            *count
        } else {
            2048
        };
        if let Some(limit) = limit {
            count = count.min(limit)
        }

        let mut datetimes = vec![start.clone()];
        let mut datetime_utc = start.utc();
        while datetimes.len() < count {
            if let Some(end) = &end {
                if datetime_utc > end.utc() {
                    break;
                }
            }
            datetimes.push(datetime_utc.into());
        }

        datetimes
    }
}

#[cfg(test)]
mod tests {
    use crate::{CalDateTime, rrule::RecurrenceRule};

    #[test]
    fn test_between() {
        let rrule = RecurrenceRule::parse("FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1").unwrap();
        let start = CalDateTime::parse("20250516T133000Z", None).unwrap();
        assert_eq!(rrule.between(start, None, Some(4)), vec![]);
    }
}
