use super::{RecurrenceFrequency, RecurrenceLimit, RecurrenceRule};
use crate::CalDateTime;
use chrono::{Datelike, Duration, IsoWeek, NaiveDate, Weekday, WeekdaySet};
use std::collections::HashSet;

/*
* https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.10
 +----------+--------+--------+-------+-------+------+-------+------+
   |          |SECONDLY|MINUTELY|HOURLY |DAILY  |WEEKLY|MONTHLY|YEARLY|
   +----------+--------+--------+-------+-------+------+-------+------+
   |BYMONTH   |Limit   |Limit   |Limit  |Limit  |Limit |Limit  |Expand|
   +----------+--------+--------+-------+-------+------+-------+------+
   |BYWEEKNO  |N/A     |N/A     |N/A    |N/A    |N/A   |N/A    |Expand|
   +----------+--------+--------+-------+-------+------+-------+------+
   |BYYEARDAY |Limit   |Limit   |Limit  |N/A    |N/A   |N/A    |Expand|
   +----------+--------+--------+-------+-------+------+-------+------+
   |BYMONTHDAY|Limit   |Limit   |Limit  |Limit  |N/A   |Expand |Expand|
   +----------+--------+--------+-------+-------+------+-------+------+
   |BYDAY     |Limit   |Limit   |Limit  |Limit  |Expand|Note 1 |Note 2|
   +----------+--------+--------+-------+-------+------+-------+------+
   |BYHOUR    |Limit   |Limit   |Limit  |Expand |Expand|Expand |Expand|
   +----------+--------+--------+-------+-------+------+-------+------+
   |BYMINUTE  |Limit   |Limit   |Expand |Expand |Expand|Expand |Expand|
   +----------+--------+--------+-------+-------+------+-------+------+
   |BYSECOND  |Limit   |Expand  |Expand |Expand |Expand|Expand |Expand|
   +----------+--------+--------+-------+-------+------+-------+------+
   |BYSETPOS  |Limit   |Limit   |Limit  |Limit  |Limit |Limit  |Limit |
   +----------+--------+--------+-------+-------+------+-------+------+

      Note 1:  Limit if BYMONTHDAY is present; otherwise, special expand
               for MONTHLY.

      Note 2:  Limit if BYYEARDAY or BYMONTHDAY is present; otherwise,
               special expand for WEEKLY if BYWEEKNO present; otherwise,
               special expand for MONTHLY if BYMONTH present; otherwise,
               special expand for YEARLY.
*/

pub(crate) fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub(crate) fn last_day_of_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => panic!("invalid month: {}", month),
    }
}

pub(crate) fn list_days_in_month(year: i32, month: u8) -> &'static [i8] {
    match last_day_of_month(year, month) {
        28 => &DAYS28,
        29 => &DAYS29,
        30 => &DAYS30,
        31 => &DAYS31,
        _ => unreachable!(),
    }
}

pub(crate) fn first_weekday(year: i32, wkst: Weekday) -> NaiveDate {
    // The first week of the year (starting from WKST) is the week having at
    // least four days in the year
    // isoweek_start marks week 1 with WKST
    let mut isoweek_start = NaiveDate::from_isoywd_opt(year, 1, wkst).unwrap();
    if isoweek_start.year() == year && isoweek_start.day0() >= 4 {
        // We can fit another week before
        isoweek_start -= Duration::days(7);
    }
    isoweek_start
}

pub(crate) fn weeks_in_year(year: i32, wkst: Weekday) -> (NaiveDate, u8) {
    let first_day = first_weekday(year, wkst);
    let next_year_first_day = first_weekday(year + 1, wkst);
    (
        first_day,
        (next_year_first_day - first_day).num_weeks() as u8,
    )
}

// Writing this bodge was easier than doing it the hard way :D
const DAYS28: [i8; 28] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28,
];
const DAYS29: [i8; 29] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29,
];
const DAYS30: [i8; 30] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30,
];
const DAYS31: [i8; 31] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31,
];

impl RecurrenceRule {
    pub fn between(
        &self,
        start: CalDateTime,
        end: Option<CalDateTime>,
        limit: Option<usize>,
    ) -> Vec<CalDateTime> {
        let mut end = end.as_ref();
        if let Some(RecurrenceLimit::Until(until)) = &self.limit {
            end = Some(end.unwrap_or(until).min(until));
        }
        let mut count = if let Some(RecurrenceLimit::Count(count)) = &self.limit {
            *count
        } else {
            2048
        };
        if let Some(limit) = limit {
            count = count.min(limit)
        }

        let datetimes = vec![];

        let mut year = start.year();
        let mut month0 = start.month0();
        // let months0 = self.bymonth0.clone().unwrap_or(vec![start.month0() as i8]);

        let offset_weekdays = self.offset_weekdays();
        let absolute_weekdays = self.absolute_weekdays();

        while datetimes.len() < count {
            let mut result_dates = vec![start.date()];
            // Iterate over frequency*interval
            match self.frequency {
                RecurrenceFrequency::Yearly => year += self.interval as i32,
                RecurrenceFrequency::Monthly => {
                    month0 += self.interval;
                    year += (month0 as f32 / 12.).floor() as i32;
                    month0 %= 12;
                }
                _ => {}
            }

            #[allow(clippy::single_match)]
            match self.frequency {
                RecurrenceFrequency::Yearly => {}
                // RecurrenceFrequency::Monthly => {
                //     // Filter bymonth
                //     if let Some(bymonth0) = &self.bymonth0 {
                //         if !bymonth0.contains(&(month0 as u8)) {
                //             continue;
                //         }
                //     }
                //
                //     if let Some(monthdays) = &self.bymonthday {
                //         for monthday in monthdays {
                //             let monthday = if *monthday > 0 {
                //                 *monthday as u32
                //             } else {
                //                 // +1 because -1 is the last day
                //                 last_day_of_month(year, month0 as u8 + 1) as u32
                //                         + 1
                //                         // monthday is negative
                //                         + *monthday as u32
                //             };
                //             let date = if let Some(date) =
                //                 NaiveDate::from_ymd_opt(year, month0 as u32 + 1, monthday)
                //             {
                //                 date
                //             } else {
                //                 continue;
                //             };
                //
                //             if let Some(weekdays) = absolute_weekdays {
                //                 if weekdays.contains(date.weekday()) {
                //                     dates.insert(date);
                //                 }
                //             } else {
                //                 dates.insert(date);
                //             }
                //         }
                //     }
                // }
                _ => {}
            }

            if let Some(end) = end {}
            // datetimes.push(datetime.to_owned());
        }

        datetimes
    }
}

#[cfg(test)]
mod tests {
    // use crate::{CalDateTime, rrule::RecurrenceRule};

    // #[test]
    // fn test_between() {
    //     // Example: Last workday of the month
    //     let rrule = RecurrenceRule::parse("FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1").unwrap();
    //     let start = CalDateTime::parse("20250516T133000Z", None).unwrap();
    //     assert_eq!(rrule.between(start, None, Some(4)), vec![]);
    // }
}
