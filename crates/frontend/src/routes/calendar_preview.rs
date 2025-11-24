use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension,
    extract::{Path, Query},
    response::{IntoResponse, Response},
};
use chrono::{
    DateTime, Datelike, Days, Duration, NaiveDate, NaiveDateTime, NaiveTime, SubsecRound, Timelike,
    Utc,
};
use chrono_tz::Tz;
use http::StatusCode;
use ical::parser::Component;
use rrule::RRuleSet;
use rustical_caldav::calendar::methods::get::get_calendar_objects;
use rustical_ical::{CalendarObjectComponent, EventObject};
use rustical_store::{CalendarStore, auth::Principal};
use serde::Deserialize;
use std::sync::Arc;
use std::{any::Any, str::FromStr};

trait CalendarPageHeader: Any {
    fn as_any(&self) -> &dyn Any;
    fn get_title(&self) -> String;
    fn get_link_next(&self) -> String;
    fn get_link_prev(&self) -> String;
    fn get_link_today(&self) -> String;
}

struct WeekDay {
    week_day: String, // name of that day
    day: String,      // day in the month
}

struct TimeSlot {
    text: String,  // time as text
    z_index: u32,  // 6 if contains text 1 otherwise
    grid_row: u32, // counter where it should be displayed
}

struct WeekEvent {
    title: String,
    date: String,
    time: String,
    margin_left: u32,    // percentage of the width of the day
    width: u32,          // percentage of the width of the day
    grid_column: u32,    // column where it should be displayed
    grid_row_start: u32, // start row: 1, 2, 3, ...
    grid_row_end: u32,   // end row_start: 1, 2, 3, ...
    z_index: u32,        // 1000 if it's the first event in a day, 1 otherwise
    bg_color_r: u8,      // background color of the event
    bg_color_g: u8,      // background color of the event
    bg_color_b: u8,      // background color of the event
}

struct CalendarPageWeek {
    title: String,
    next: String,
    prev: String,
    today: String,

    days: Vec<WeekDay>,
    timeslots: Vec<TimeSlot>,
    events: Vec<WeekEvent>,
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/calendar_preview.html")]
struct CalendarPage<T: CalendarPageHeader> {
    content: T,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
enum ViewOptions {
    #[serde(alias = "d", alias = "day")]
    Day,
    #[serde(alias = "w", alias = "week")]
    Week,
    #[serde(alias = "m", alias = "month")]
    Month,
    #[serde(alias = "y", alias = "year")]
    Year,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreviewQuery {
    view: Option<ViewOptions>, // defaults to week view
    #[allow(dead_code)]
    week: Option<u32>, // number of the week in the week view
    #[allow(dead_code)]
    month: Option<u32>, // number of the month in the month view
    #[allow(dead_code)]
    year: Option<u32>, // number of the year in year overview
    start: Option<NaiveDate>,  // date range if a specific date range should be displayed
    end: Option<NaiveDate>,
    limit: Option<usize>,         // limit the number of events to display
    color_schema: Option<String>, // color schema to use for the events
}

#[derive(Debug)]
pub struct DisplayEvent {
    #[allow(dead_code)]
    pub uid: String,
    #[allow(dead_code)]
    pub recurrence_id: Option<String>,
    pub summary: Option<String>,
    #[allow(dead_code)]
    pub rrule: Option<String>,
    pub dtstart: DateTime<Tz>,
    pub dtend: DateTime<Tz>,
}

impl CalendarPageHeader for CalendarPageWeek {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_title(&self) -> String {
        return self.title.clone();
    }

    fn get_link_next(&self) -> String {
        return self.next.clone();
    }

    fn get_link_prev(&self) -> String {
        return self.prev.clone();
    }

    fn get_link_today(&self) -> String {
        return self.today.clone();
    }
}

// Helper function for handling overlapping events
fn set_at_index_or_resize<T>(vec: &mut Vec<T>, index: usize, value: T)
where
    T: Clone,
{
    if index >= vec.len() {
        vec.resize(index + 1, value);
    } else {
        vec[index] = value;
    }
}

pub async fn preview_calendar<C: CalendarStore>(
    Path((owner, cal_id)): Path<(String, String)>,
    Query(query): Query<PreviewQuery>,
    Extension(store): Extension<Arc<C>>,
    user: Principal,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    if query.view.is_some() && query.view.unwrap() != ViewOptions::Week {
        return Ok(StatusCode::NOT_IMPLEMENTED.into_response());
    }

    let calendar = store.get_calendar(&owner, &cal_id, true).await?;
    let show_in_timezone = calendar.get_timezone().unwrap_or_default();
    let color = calendar.meta.color;

    let start: DateTime<Tz>;
    let end: DateTime<Tz>;
    // max timerange is 1 year
    if let (Some(start_p), Some(end_p)) = (query.start, query.end) {
        if end_p < start_p {
            tracing::warn!("The end must be after the start date");
            return Ok(StatusCode::BAD_REQUEST.into_response());
        } else if end_p > start_p.checked_add_signed(Duration::days(365)).unwrap() {
            tracing::warn!("The timerange must not exceed 1 year");
            return Ok(StatusCode::BAD_REQUEST.into_response());
        }
        start = start_p
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_local_timezone(show_in_timezone)
            .unwrap();
        end = end_p
            .and_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap())
            .and_local_timezone(show_in_timezone)
            .unwrap();
    } else {
        // default to the current week
        let now = Utc::now().with_timezone(&show_in_timezone);
        let start_of_week = now - Duration::days(now.weekday().num_days_from_monday() as i64);
        start = start_of_week
            .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .unwrap();
        end = (start_of_week + Duration::days(6))
            .with_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap())
            .unwrap();
    }

    let calendar = get_calendar_objects::<C>(
        &owner,
        &cal_id,
        start.date_naive(),
        end.date_naive(),
        &store,
    )
    .await
    .expect("Could not retrive calendar object.");

    let mut eventdates: Vec<DisplayEvent> = Vec::new();

    for object in &calendar {
        match object.get_data() {
            CalendarObjectComponent::Event(EventObject { event, .. }, ..) => {
                let event_uid = event.get_property("UID");
                let event_starttime_prop = event.get_property("DTSTART");
                let event_endtime_prop = event.get_property("DTEND");
                let event_rrule = event.get_property("RRULE");
                let event_exrule = event.get_property("EXRULE");
                let event_recurrence_id = event.get_property("RECURRENCE-ID");
                let event_title = event.get_property("SUMMARY");

                // Parse Datetime
                if let (Some(starttime_prop), Some(endtime_prop)) =
                    (event_starttime_prop, event_endtime_prop)
                {
                    let starttime_zone = starttime_prop.get_param("TZID").unwrap();
                    let starttime_str = starttime_prop.clone().value.unwrap();
                    let endtime_zone = endtime_prop.get_param("TZID").unwrap();
                    let endtime_str = endtime_prop.clone().value.unwrap();

                    let tz_start: Tz = starttime_zone
                        .parse()
                        .expect("Failed to parse start timezone");
                    let starttime: DateTime<Tz> =
                        NaiveDateTime::parse_from_str(&starttime_str, "%Y%m%dT%H%M%S")
                            .or_else(|_| {
                                NaiveDateTime::parse_from_str(&starttime_str, "%Y-%m-%dT%H:%M:%S%z")
                            })
                            .expect("Failed to parse start datetime")
                            .and_local_timezone(tz_start)
                            .earliest()
                            .unwrap()
                            .with_timezone(&show_in_timezone);
                    let tz_end: Tz = endtime_zone.parse().expect("Failed to parse end timezone");
                    let endtime: DateTime<Tz> =
                        NaiveDateTime::parse_from_str(&endtime_str, "%Y%m%dT%H%M%S")
                            .or_else(|_| {
                                NaiveDateTime::parse_from_str(&endtime_str, "%Y-%m-%dT%H:%M:%S%z")
                            })
                            .expect("Failed to parse end datetime")
                            .and_local_timezone(tz_end)
                            .earliest()
                            .unwrap()
                            .with_timezone(&show_in_timezone);

                    let event_uid = match event_uid {
                        Some(uid) => uid.clone().value.unwrap_or_default(),
                        None => "".to_string(),
                    };
                    let event_title = match event_title {
                        Some(title) => title.clone().value,
                        None => None,
                    };

                    let mut rrule_string = String::new();

                    if let Some(event_rrule) = event_rrule
                        && let Some(event_rrule) = event_rrule.clone().value
                    {
                        // is a recurring event
                        if let Some(event_exrule) = event_exrule
                            && let Some(event_exrule) = event_exrule.clone().value
                        {
                            rrule_string += &format!("\nEXRULE:{}", event_exrule);
                        }
                        if let Some(event_exdate) = event.get_property("EXDATE")
                            && let Some(event_exdate) = event_exdate.clone().value
                        {
                            rrule_string += &format!("\nEXDATE:{}", event_exdate);
                        }

                        let rrule: RRuleSet = format!(
                            "DTSTART;TZID={}:{}\nRRULE:{}{}",
                            starttime.timezone(),
                            starttime.format("%Y%m%dT%H%M%S"),
                            event_rrule,
                            rrule_string,
                        )
                        .parse()
                        .expect("Failed to parse rrule");

                        let limit: u16 = query
                            .limit
                            .unwrap_or(100)
                            .try_into()
                            .expect("Could not convert number format for limit");

                        let start_rrule: DateTime<rrule::Tz> = DateTime::from_timestamp(
                            start.with_timezone(&Utc).timestamp(),
                            start.with_timezone(&Utc).timestamp_subsec_nanos(),
                        )
                        .unwrap()
                        .with_timezone(&rrule::Tz::UTC);
                        let event_dates = rrule.clone().after(start_rrule).all(limit).dates;

                        let event_recurrence_id = match event_recurrence_id {
                            Some(recurrence_id) => {
                                Some(recurrence_id.clone().value.unwrap_or_default().clone())
                            }
                            None => None,
                        };

                        for date in event_dates {
                            // create entries for all sub events

                            let date: DateTime<Tz> = DateTime::from_timestamp(
                                date.timestamp(),
                                date.timestamp_subsec_nanos(),
                            )
                            .unwrap()
                            .with_timezone(&Tz::from_str(&date.timezone().to_string()).unwrap());

                            let startdate = date.with_timezone(&show_in_timezone);
                            let enddate = startdate + (endtime - starttime);

                            let event = DisplayEvent {
                                uid: event_uid.clone(),
                                rrule: Some(event_rrule.clone()),
                                recurrence_id: event_recurrence_id.clone(),
                                summary: event_title.clone(),
                                dtstart: startdate,
                                dtend: enddate,
                            };
                            eventdates.push(event);
                        }
                    } else {
                        // is only a one time event
                        let event = DisplayEvent {
                            uid: event_uid,
                            rrule: None,
                            recurrence_id: None,
                            summary: event_title,
                            dtstart: starttime,
                            dtend: endtime,
                        };
                        eventdates.push(event);
                    }
                }
            }
            _ => continue,
        }
    }

    // sort all by timestamp
    eventdates.sort_by(|a, b| a.dtstart.cmp(&b.dtstart));

    // remove all events outside the date range
    eventdates.retain(|event| {
        // this does not work for events that start before the timeframe but end inside
        event.dtend >= start
            && event.dtstart
                < end
                    .checked_add_days(Days::new(1))
                    .expect("Day could not be added")
                    .into()
    });

    // compute overlaps
    let mut overlaps = Vec::<(DateTime<Tz>, DateTime<Tz>, u8)>::new();
    for event in &eventdates {
        // round to nearest 15 minutes
        let date_start = event.dtstart.round_subsecs(15 * 60);
        let date_end = event.dtend.round_subsecs(15 * 60);
        let mut event_overlaps = 1;
        for (idx, (other_start, other_end, other_overlaps)) in overlaps.clone().iter().enumerate() {
            if date_start < *other_end && date_end > *other_start {
                // overlap
                event_overlaps += 1;
                overlaps[idx].2 = other_overlaps + 1;
            }
        }
        overlaps.push((date_start, date_end, event_overlaps));
    }

    // function to generate the html code for the week view
    let (days, timeslots, title, next_link, prev_link, today_link) =
        generate_html_header_week_view(start, end, None, query.color_schema.clone());
    let events = generate_html_week_view(
        eventdates,
        overlaps,
        start,
        end,
        None,
        &show_in_timezone,
        query.color_schema,
        color,
    );

    Ok(CalendarPage {
        content: CalendarPageWeek {
            days: days,
            timeslots: timeslots,
            events: events,
            title: title,
            next: next_link,
            prev: prev_link,
            today: today_link,
        },
    }
    .into_response())
}

/// Generate the header bar for the week view
fn generate_html_header_week_view(
    start: DateTime<Tz>,
    end: DateTime<Tz>,
    steps_per_hour: Option<u8>,
    color_schema: Option<String>,
) -> (Vec<WeekDay>, Vec<TimeSlot>, String, String, String, String) {
    let mut days: Vec<WeekDay> = Vec::new();
    let mut timeslots: Vec<TimeSlot> = Vec::new();

    let start_time: NaiveTime = start.time();
    let end_time: NaiveTime = end.time();
    let start: NaiveDate = start.date_naive();
    let end: NaiveDate = end.date_naive();

    let days_diff = end.signed_duration_since(start).num_days() as u32;
    for i in 0..days_diff + 1 {
        days.push(WeekDay {
            week_day: start
                .checked_add_days(Days::new(i.into()))
                .unwrap()
                .format("%A")
                .to_string(),
            day: start
                .checked_add_days(Days::new(i.into()))
                .unwrap()
                .format("%d")
                .to_string(),
        });
    }

    let hours_diff = end_time.signed_duration_since(start_time).num_hours() as u32;
    let steps_per_hour = steps_per_hour.unwrap_or(4);
    let mut counter = 2;
    for i in 0..hours_diff + 1 {
        let time = start_time + Duration::hours(i as i64);
        for step in 0..steps_per_hour {
            let minutes = (60 / steps_per_hour) * step;
            let time_str = if step == 0 {
                format!("{}:{:02}", time.hour(), minutes)
            } else {
                "".to_string()
            };
            let z_index = if step == 0 { 1 } else { 0 };
            timeslots.push(TimeSlot {
                text: time_str,
                z_index: z_index,
                grid_row: counter,
            });
            counter += 1;
        }
    }

    let title = format!("Week {}: {}", start.format("%U"), start.format("%B %d, %Y"));

    let button_next = format!(
        r#"preview?view=Week&start={next_week_start}&end={next_week_end}&color_schema={color}"#,
        next_week_start = start
            .checked_add_days(Days::new(7))
            .unwrap()
            .format("%Y-%m-%d")
            .to_string(),
        next_week_end = end
            .checked_add_days(Days::new(7))
            .unwrap()
            .format("%Y-%m-%d")
            .to_string(),
        color = color_schema.clone().unwrap_or("".to_string())
    );

    let button_prev = format!(
        r#"preview?view=Week&start={prev_week_start}&end={prev_week_end}&color_schema={color}"#,
        prev_week_start = start
            .checked_sub_days(Days::new(7))
            .unwrap()
            .format("%Y-%m-%d")
            .to_string(),
        prev_week_end = end
            .checked_sub_days(Days::new(7))
            .unwrap()
            .format("%Y-%m-%d")
            .to_string(),
        color = color_schema.clone().unwrap_or("".to_string())
    );

    let button_today = format!(
        r#"preview?view=Week&color_schema={color}"#,
        color = color_schema.clone().unwrap_or("".to_string())
    );

    return (
        days,
        timeslots,
        title,
        button_next,
        button_prev,
        button_today,
    );
}

/// Generate the HTML for the week view
/// The data needs to be sorted by start time in acending order
fn generate_html_week_view(
    events: Vec<DisplayEvent>,
    overlaps: Vec<(DateTime<Tz>, DateTime<Tz>, u8)>,
    start: DateTime<Tz>,
    end: DateTime<Tz>,
    steps_per_hour: Option<u8>,
    render_timezone: &Tz,
    color_schema: Option<String>,
    default_color: Option<String>,
) -> Vec<WeekEvent> {
    let mut display_events = Vec::new();
    let mut overlaps_map = Vec::<DateTime<Tz>>::new();

    let start_date: NaiveDate = start.date_naive();
    let end_date: NaiveDate = end.date_naive();
    let start_time: NaiveTime = start.time();
    let end_time: NaiveTime = end.time();

    for event in events {
        if event.dtstart > event.dtend {
            continue;
        }

        // compute the position
        let rounded_start = event.dtstart.round_subsecs(15 * 60);
        let rounded_end = event.dtend.round_subsecs(15 * 60);
        let overlap_count = overlaps
            .iter()
            .find(|e| e.0 == rounded_start && e.1 == rounded_end)
            .expect("Could not find Timeslot");
        // find a free entry
        let overlaps_map_index = overlaps_map
            .iter()
            .position(|&x| x <= rounded_start)
            .unwrap_or(overlaps_map.len());
        set_at_index_or_resize(&mut overlaps_map, overlaps_map_index, rounded_end);
        let mut left = (((overlaps_map_index) as f32) * 100.0 / (overlap_count.2 as f32)) as i32;
        if left == 100 || left == 0 {
            left = 0;
        }

        let start_hour = rounded_start.with_timezone(render_timezone).hour();
        let start_minute = rounded_start.with_timezone(render_timezone).minute();
        let end_hour = rounded_end.with_timezone(render_timezone).hour();
        let end_minute = rounded_end.with_timezone(render_timezone).minute();

        // Todo: if the event goes over multiple days needs to be split and represented by multiple bars
        let mut days = chrono::Duration::zero();
        if rounded_start.date_naive() != rounded_end.date_naive() {
            days = rounded_end.date_naive() - rounded_start.date_naive();
        }

        // only valid for first day
        let start_whole_row = (start_time.hour() as u32) * steps_per_hour.unwrap_or(4) as u32
            + ((start_time.minute() as f32) / (60 / steps_per_hour.unwrap_or(4)) as f32).round()
                as u32;
        let end_whole_row = (end_time.hour() as u32) * steps_per_hour.unwrap_or(4) as u32
            + ((end_time.minute() as f32) / (60 / steps_per_hour.unwrap_or(4)) as f32).round()
                as u32
            + 2;
        let start_row = (start_hour as u32) * steps_per_hour.unwrap_or(4) as u32
            + ((start_minute as f32) / (60 / steps_per_hour.unwrap_or(4)) as f32).round() as u32;

        let title = event.summary.clone().unwrap_or_default();
        // let title_escaped = escape(&title, Html).to_string();

        let (random_hex_color_r, random_hex_color_g, random_hex_color_b) = match color_schema {
            Some(ref s) if s == "random" => {
                let random_hex_color_r = rand::random::<u8>();
                let random_hex_color_g = rand::random::<u8>();
                let random_hex_color_b = rand::random::<u8>();
                (random_hex_color_r, random_hex_color_g, random_hex_color_b)
            }
            Some(ref s) if s == "tableu10" => {
                // Use a color from the Tableau 10 color palette
                let colors = [
                    (31, 119, 180),
                    (255, 127, 14),
                    (44, 160, 44),
                    (214, 39, 40),
                    (148, 103, 189),
                    (140, 86, 75),
                    (227, 119, 194),
                    (127, 127, 127),
                    (188, 189, 34),
                    (23, 190, 207),
                ];
                let random_color = colors[(overlaps_map_index % colors.len()) as usize];
                random_color
            }
            _ => {
                // Default color
                let mut r = 50;
                let mut g = 50;
                let mut b = 50; // Default color if no color is specified
                if let Some(color) = default_color.clone() {
                    // Parse the color string to RGB values
                    let color = color.strip_prefix('#').unwrap_or(&color);
                    r = u8::from_str_radix(&color[0..2], 16).unwrap_or(0);
                    g = u8::from_str_radix(&color[2..4], 16).unwrap_or(0);
                    b = u8::from_str_radix(&color[4..6], 16).unwrap_or(0);
                }
                (r, g, b)
            }
        };

        // iterate over all days that the event has
        for i in 0..(days.num_days() + 1) {
            let current_day = rounded_start
                .checked_add_days(Days::new(i as u64))
                .expect("Date overflow")
                .with_timezone(render_timezone);
            if current_day.date_naive() < start_date || current_day.date_naive() > end_date {
                continue;
            }

            let mut start_row = (start_row as i32 - start_whole_row as i32) as i32 + 2;
            let end_row = ((end_hour as u32) * steps_per_hour.unwrap_or(4) as u32
                + ((end_minute as f32) / (60 / steps_per_hour.unwrap_or(4)) as f32).round() as u32)
                as i32;
            let mut end_row = (end_row as i32 - start_whole_row as i32) as i32 + 2;
            // if it is not the first day update the start row
            if i > 0 || start_row < 2 {
                start_row = 2;
            }
            if i < days.num_days() {
                end_row = end_whole_row as i32;
            } else {
                if end_row < 2 {
                    continue;
                }
                if end_row > end_whole_row as i32 {
                    end_row = end_whole_row as i32;
                }
            }

            display_events.push(WeekEvent {
                title: title.clone(),
                date: format!("{} {}", current_day.format("%b"), current_day.day()),
                time: format!(
                    "{:02}:{:02} - {:02}:{:02}",
                    start_hour, start_minute, end_hour, end_minute
                ),
                margin_left: left as u32,
                width: (100 - left) as u32,
                grid_column: (current_day.weekday().number_from_monday() - 1) % 7 + 2,
                grid_row_start: start_row as u32,
                grid_row_end: end_row as u32,
                z_index: overlaps_map_index as u32,
                bg_color_r: random_hex_color_r,
                bg_color_g: random_hex_color_g,
                bg_color_b: random_hex_color_b,
            });
        }
    }
    display_events
}
