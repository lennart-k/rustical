use crate::calendar::CalDateTime;

use super::{RecurrenceLimit, RecurrenceRule};

impl RecurrenceRule {
    pub fn between(
        &self,
        start: CalDateTime,
        end: Option<CalDateTime>,
    ) -> impl IntoIterator<Item = CalDateTime> {
        let start = start.cal_utc();
        // Terrible code, should clean this up later.
        let mut end = end.map(|end| CalDateTime::cal_utc(&end));
        if let Some(RecurrenceLimit::Until(until)) = &self.limit {
            let until = until.cal_utc();
            let mut _end = end.unwrap_or(until.clone());
            if until.utc() < _end.utc() {
                _end = until;
            }
            end = Some(_end);
        }
        let count = if let Some(RecurrenceLimit::Count(count)) = &self.limit {
            *count
        } else {
            2048
        };

        let mut datetimes = vec![start.clone()];
        let mut datetime_utc = start.utc();
        while datetimes.len() < count {
            if let Some(end) = &end {
                if datetime_utc > end.utc() {
                    break;
                }
                datetimes.push(CalDateTime::Utc(datetime_utc));
            }
        }

        datetimes
    }
}
