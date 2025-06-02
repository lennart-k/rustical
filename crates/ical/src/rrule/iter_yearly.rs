use std::collections::HashSet;

use chrono::{Datelike, Duration, NaiveDate};

use super::{
    RecurrenceRule,
    iter::{last_day_of_month, list_days_in_month, weeks_in_year},
};

impl RecurrenceRule {
    pub fn week_expansion(&self, year: i32) -> Option<HashSet<NaiveDate>> {
        let absolute_weekdays = self.absolute_weekdays();

        if let Some(byweekno) = &self.byweekno {
            let mut dates = HashSet::new();

            let weekstart = self.week_start.unwrap_or(chrono::Weekday::Mon);
            let (first_weekstart, num_weeks) = weeks_in_year(year, weekstart);

            let weeknums0: Vec<u8> = byweekno
                .iter()
                .map(|num| {
                    if *num < 0 {
                        (num_weeks as i8 + *num) as u8
                    } else {
                        (*num - 1) as u8
                    }
                })
                .collect();

            for weeknum0 in weeknums0 {
                let weekstart_date = first_weekstart + Duration::weeks(weeknum0 as i64);
                // Iterate over the week and check if the weekdays are allowed
                for i in 0..7 {
                    let date = weekstart_date + Duration::days(i);
                    if let Some(weekdays) = absolute_weekdays {
                        if weekdays.contains(date.weekday()) {
                            dates.insert(date);
                        }
                    } else {
                        dates.insert(date);
                    }
                }
            }
            Some(dates)
        } else {
            None
        }
    }

    pub fn month_expansion(&self, year: i32) -> Option<HashSet<NaiveDate>> {
        let offset_weekdays = self.offset_weekdays();
        let absolute_weekdays = self.absolute_weekdays();

        if let Some(bymonth0) = &self.bymonth0 {
            let mut dates = HashSet::new();
            for month0 in bymonth0 {
                // Add BYMONTHDAY or all days
                let monthdays = self
                    .bymonthday
                    .as_deref()
                    .unwrap_or(list_days_in_month(year, month0 + 1));

                for monthday in monthdays {
                    let monthday = if *monthday > 0 {
                        *monthday as u32
                    } else {
                        // +1 because -1 is the last day
                        last_day_of_month(year, month0 + 1) as u32
                                        + 1
                                        // monthday is negative
                                        + *monthday as u32
                    };
                    let date = if let Some(date) =
                        NaiveDate::from_ymd_opt(year, *month0 as u32 + 1, monthday)
                    {
                        date
                    } else {
                        continue;
                    };

                    if let Some(weekdays) = absolute_weekdays {
                        if weekdays.contains(date.weekday()) {
                            dates.insert(date);
                        }
                    } else {
                        dates.insert(date);
                    }
                }

                // Add offset weekdays
                if let Some(offset_weekdays) = &offset_weekdays {
                    for (num, day) in offset_weekdays.iter() {
                        let date = if *num > 0 {
                            NaiveDate::from_weekday_of_month_opt(
                                year,
                                *month0 as u32 + 1,
                                *day,
                                *num as u8,
                            )
                        } else {
                            // If index negative:
                            // Go to first day of next month and walk back the weeks
                            NaiveDate::from_weekday_of_month_opt(
                                year,
                                *month0 as u32 + 1 + 1,
                                *day,
                                1,
                            )
                            .map(|date| date + Duration::weeks(*num))
                        };

                        if let Some(date) = date {
                            dates.insert(date);
                        }
                    }
                }
            }
            Some(dates)
        } else {
            None
        }
    }

    pub fn dates_yearly(
        &self,
        start: NaiveDate,
        end: Option<NaiveDate>,
        limit: usize,
    ) -> Vec<NaiveDate> {
        let mut dates = vec![start];
        let mut year = start.year();

        while dates.len() < limit {
            // Expand BYMONTH
            let month_expansion = self.month_expansion(year);

            // Expand BYWEEKNO
            let week_expansion = self.week_expansion(year);

            let mut occurence_set = match (month_expansion, week_expansion) {
                (Some(month_expansion), Some(week_expansion)) => month_expansion
                    .intersection(&week_expansion)
                    .cloned()
                    .collect(),
                (Some(month_expansion), None) => month_expansion,
                (None, Some(week_expansion)) => week_expansion,
                (None, None) => start
                    .with_year(year)
                    .map(|date| HashSet::from([date]))
                    .unwrap_or_default(),
            }
            .into_iter()
            .collect::<Vec<_>>();
            occurence_set.sort();
            if let Some(bysetpos) = &self.bysetpos {
                occurence_set = bysetpos
                    .iter()
                    .filter_map(|i| {
                        if *i > 0 {
                            occurence_set.get((*i - 1) as usize)
                        } else {
                            occurence_set.get((occurence_set.len() as i64 + *i) as usize)
                        }
                    })
                    .cloned()
                    .collect();
            }
            dates.extend_from_slice(occurence_set.as_slice());
            year += 1;
        }

        dates
    }
}
