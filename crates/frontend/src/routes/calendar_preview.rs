use std::sync::Arc;

use askama::Template;
use askama_escape::{Html, escape};
use askama_web::WebTemplate;
use axum::{
    Extension,
    extract::{Path, Query},
    response::{IntoResponse, Response},
};
use chrono::{Datelike, Days, Duration, NaiveDate, NaiveDateTime, SecondsFormat, SubsecRound, Timelike, Utc};
use http::StatusCode;
use rrule::RRuleSet;
use rustical_caldav::calendar::methods::get::get_calendar_objects;
use rustical_store::{CalendarStore, auth::Principal};
use ical::generator::{IcalCalendar, IcalCalendarBuilder};
use serde::Deserialize;

#[derive(Template, WebTemplate)]
#[template(path = "pages/calendar_preview.html")]
struct CalendarPage {
    header_days: String,
    title: String,
    next: String,
    prev: String,
    today: String,
    events: String,
}

#[derive(Debug, Clone, Deserialize)]
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
    week: Option<u32>, // number of the week in the week view
    month: Option<u32>, // number of the month in the month view
    year: Option<u32>, // number of the year in year overview
    start: Option<NaiveDate>, // date range if a specific date range should be displayed
    end: Option<NaiveDate>,
    limit: Option<usize>, // limit the number of events to display
}

#[derive(Debug)]
pub struct DisplayEvent {
    pub uid: String,
    pub recurrence_id: Option<String>,
    pub summary: Option<String>,
    pub rrule: Option<String>,
    pub dtstart: String,
    pub dtend: String,
}

// Helper function for handling overlapping events
fn set_at_index_or_resize<T>(vec: &mut Vec<T>, index: usize, value: T) 
where 
    T: Default + Clone,
{
    if index >= vec.len() {
        vec.resize(index + 1, T::default());
    }
    vec[index] = value;
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

    let start: NaiveDate;
    let end: NaiveDate;
    // max timerange is 1 year
    if query.start.is_some() && query.end.is_some() {
        if query.end.unwrap() < query.start.unwrap() {
            tracing::warn!("The end must be after the start date");
            return Ok(StatusCode::BAD_REQUEST.into_response());
        } else if query.end.unwrap()
            > query
                .start
                .unwrap()
                .checked_add_signed(Duration::days(365))
                .unwrap()
        {
            tracing::warn!("The timerange must not exceed 1 year");
            return Ok(StatusCode::BAD_REQUEST.into_response());
        }
        start = query.start.unwrap();
        end = query.end.unwrap();
    } else {
        // default to the current week
        let now = Utc::now().date_naive();
        let start_of_week = now - Duration::days(now.weekday().num_days_from_monday() as i64);
        start = start_of_week;
        end = start_of_week + Duration::days(6);
    }

    let calendar_obj = get_calendar_objects::<C>(&owner, &cal_id, start, end, &store)
        .await;
    let calendar: IcalCalendar;
    if calendar_obj.is_err() {
        calendar = IcalCalendarBuilder::version("4.0").gregorian()
            .prodid("RustiCal").build().expect("Failed to create empty calendar");
    } else {
        calendar = calendar_obj.unwrap();
    }

    let mut eventdates: Vec<DisplayEvent> = Vec::new();

    // Get all event dates in the calendar
    calendar.events.iter().for_each(|event| {
        // get all events that are in a given time range
        let mut is_recurrent = 0;
        let mut event_uid = String::new();
        let mut event_rrule = String::new();
        let mut event_recurrence_id = String::new();
        let mut event_summary = String::new();
        let mut event_dtstart = String::new();
        let mut event_dtend = String::new();

        // Todo implement the exclusion field as well
        event.properties.iter().for_each(|prop| {
            // list id
            if prop.name == "UID" {
                event_uid = prop.value.clone().unwrap();
                is_recurrent += 1;
            };
            // reccurrence rule
            if prop.name == "RRULE" {
                event_rrule = prop.value.clone().unwrap();
                is_recurrent += 1;
            };
            // recurence id
            if prop.name == "RECURRENCE-ID" {
                event_recurrence_id = prop.value.clone().unwrap();
            };
            // start date
            if prop.name == "DTSTART" {
                event_dtstart = prop.value.clone().unwrap();
                is_recurrent += 1;
            };
            // end date
            if prop.name == "DTEND" {
                event_dtend = prop.value.clone().unwrap();
            };
            // title
            if prop.name == "SUMMARY" {
                event_summary = prop.value.clone().unwrap();
            };
        });

        if is_recurrent < 3 {
            // a single event
            let event = DisplayEvent {
                uid: event_uid,
                rrule: None,
                recurrence_id: None,
                summary: Some(event_summary),
                dtstart: event_dtstart,
                dtend: event_dtend,
            };
            eventdates.push(event);
        } else {
            // a recurring event
            let rrule: RRuleSet = format!("DTSTART:{}\nRRULE:{}", event_dtstart, event_rrule)
                .parse()
                .expect("Failed to parse rrule");
            let limit: u16 = query
                .limit
                .unwrap_or(100)
                .try_into()
                .expect("Could not convert number format for limit");
            let event_dates = rrule.all(limit).dates;
            for date in event_dates {
                // create entries for all sub events
                let event_end = date
                    + (NaiveDateTime::parse_from_str(&event_dtend, "%Y%m%dT%H%M%S")
                        .expect("Failed to parse date end date")
                        - NaiveDateTime::parse_from_str(&event_dtstart, "%Y%m%dT%H%M%S")
                            .expect("Failed to parse date start date"));

                let event = DisplayEvent {
                    uid: event_uid.clone(),
                    rrule: Some(event_rrule.clone()),
                    recurrence_id: Some(event_recurrence_id.clone()),
                    summary: Some(event_summary.clone()),
                    dtstart: date.to_rfc3339_opts(SecondsFormat::Secs, true),
                    dtend: event_end.to_rfc3339_opts(SecondsFormat::Secs, true),
                };
                eventdates.push(event);
            }
        }
    });

    // sort all by timestamp
    eventdates.sort_by(|a, b| {
        NaiveDateTime::parse_from_str(&a.dtstart, "%Y-%m-%dT%H:%M:%S%z")
            .or_else(|_| NaiveDateTime::parse_from_str(&a.dtstart, "%Y%m%dT%H%M%S"))
            .expect("Failed to parse date")
            .cmp(
                &NaiveDateTime::parse_from_str(&b.dtstart, "%Y-%m-%dT%H:%M:%S%z")
                    .or_else(|_| NaiveDateTime::parse_from_str(&b.dtstart, "%Y%m%dT%H%M%S"))
                    .expect("Failed to parse date"),
            )
        }
    );
    // remove all events outside the date range
    eventdates.retain(|event| {
        let event_date: NaiveDateTime;
        if let Ok(date) =
            NaiveDateTime::parse_from_str(&event.dtstart, "%Y-%m-%dT%H:%M:%S%z")
        {
            event_date = date;
        } else if let Ok(date) =
            NaiveDateTime::parse_from_str(&event.dtstart, "%Y%m%dT%H%M%S")
        {
            event_date = date;
        } else {
            tracing::warn!("Could not parse date: {:?}", event.dtstart);
            return false;
        };

        let event_end_date: NaiveDateTime;
        if let Ok(date) =
            NaiveDateTime::parse_from_str(&event.dtend, "%Y-%m-%dT%H:%M:%S%z")
        {
            event_end_date = date;
        } else if let Ok(date) =
            NaiveDateTime::parse_from_str(&event.dtend, "%Y%m%dT%H%M%S")
        {
            event_end_date = date;
        } else {
            tracing::warn!("Could not parse date: {:?}", event.dtend);
            return false;
        };

        // this does not work for events that start before the timeframe but end inside
        event_end_date >= start.into()
            && event_date
                < end
                    .checked_add_days(Days::new(1))
                    .expect("Day could not be added")
                    .into()
    });

    // compute overlaps
    let mut overlaps = Vec::<(NaiveDateTime, NaiveDateTime, u8)>::new();
    for event in &eventdates {
        let date_start =
            NaiveDateTime::parse_from_str(&event.dtstart, "%Y-%m-%dT%H:%M:%S%z")
                .or_else(|_| NaiveDateTime::parse_from_str(&event.dtstart, "%Y%m%dT%H%M%S"))
                .expect("Failed to parse date");
        let date_end = NaiveDateTime::parse_from_str(&event.dtend, "%Y-%m-%dT%H:%M:%S%z")
                .or_else(|_| NaiveDateTime::parse_from_str(&event.dtend, "%Y%m%dT%H%M%S"))
                .expect("Failed to parse date");
        // round to nearest 15 minutes
        let date_start = date_start.round_subsecs(15 * 60);
        let date_end = date_end.round_subsecs(15 * 60);
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

    // function to generate the html code for week and month views
    let (title, header_html, button_next, button_prev) = generate_html_header_week_view(start, end, &cal_id);
    let events_html = generate_html_week_view(eventdates, overlaps, start, end);

    Ok(CalendarPage {
        header_days: header_html,
        title: title,
        next: button_next,
        prev: button_prev,
        today: cal_id.to_string(),
        events: events_html,
    }
    .into_response())
}

/// Generate the header bar for the week view
fn generate_html_header_week_view(start: NaiveDate, end: NaiveDate, cal_id: &str) -> (String, String, String, String) {
    let mut html = String::new();

    html.push_str(&format!(
        r#"<div class="day-header">Time<br><span></span></div>"#
    ));

    let mut current_date = start;
    while current_date <= end {
        html.push_str(&format!(
            r#"<div class="day-header">{week_day}<br><span>{day}</span></div>"#,
            week_day = current_date.format("%A").to_string(),
            day = current_date.format("%d").to_string()
        ));
        current_date = current_date.checked_add_days(Days::new(1)).unwrap();
    }
    let title = format!("Week {}: {}",start.format("%U"), start.format("%B %d, %Y"));

    let button_next = format!(
        r#"{cal_id}?view=Week&start={next_week_start}&end={next_week_end}"#,
        next_week_start = start.checked_add_days(Days::new(7)).unwrap().format("%Y-%m-%d").to_string(),
        next_week_end = end.checked_add_days(Days::new(7)).unwrap().format("%Y-%m-%d").to_string()
    );

    let button_prev = format!(
        r#"{cal_id}?view=Week&start={prev_week_start}&end={prev_week_end}"#,
        prev_week_start = start.checked_sub_days(Days::new(7)).unwrap().format("%Y-%m-%d").to_string(),
        prev_week_end = end.checked_sub_days(Days::new(7)).unwrap().format("%Y-%m-%d").to_string()
    );

    return (title, html, button_next, button_prev);
}

/// Generate the HTML for the week view
/// The data needs to be sorted by start time in acending order
fn generate_html_week_view(
    events: Vec<DisplayEvent>,
    overlaps: Vec<(chrono::NaiveDateTime, chrono::NaiveDateTime, u8)>,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
) -> String {
    let mut html = String::new();
    let mut overlaps_map = Vec::<chrono::NaiveDateTime>::new();

    for event in events {
        let event_date_start: NaiveDateTime;
        if let Ok(date) =
            NaiveDateTime::parse_from_str(&event.dtstart, "%Y-%m-%dT%H:%M:%S%z")
        {
            event_date_start = date;
        } else if let Ok(date) =
            NaiveDateTime::parse_from_str(&event.dtstart, "%Y%m%dT%H%M%S")
        {
            event_date_start = date;
        } else {
            tracing::warn!("Could not parse date: {:?}", event.dtstart);
            return "".to_string();
        };

        let event_date_end: NaiveDateTime;
        if let Ok(date) = NaiveDateTime::parse_from_str(&event.dtend, "%Y-%m-%dT%H:%M:%S%z")
        {
            event_date_end = date;
        } else if let Ok(date) =
            NaiveDateTime::parse_from_str(&event.dtend, "%Y%m%dT%H%M%S")
        {
            event_date_end = date;
        } else {
            tracing::warn!("Could not parse date: {:?}", event.dtend);
            return "".to_string();
        };

        // compute the position
        let rounded_start = event_date_start.round_subsecs(15 * 60);
        let rounded_end = event_date_end.round_subsecs(15 * 60);
        let overlap_count = overlaps.iter().find(|e| e.0 == event_date_start && e.1 == event_date_end).expect("Could not find Timeslot");
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


        let start_hour = rounded_start.hour();
        let start_minute = rounded_start.minute();
        let end_hour = rounded_end.hour();
        let end_minute = rounded_end.minute();

        // Todo: if the event goes over multiple days needs to be split and represented by multiple bars
        let mut days = chrono::Duration::zero();
        if rounded_start.date() != rounded_end.date() {
            days = rounded_end.date() - rounded_start.date();
        }

        // only valid for first day
        let mut start_row = (start_hour * 4) + (start_minute / 15) + 2;
        let mut end_row: u32;

        let random_hex_color_r = rand::random::<u8>();
        let random_hex_color_g = rand::random::<u8>();
        let random_hex_color_b = rand::random::<u8>();

        // iterate over all days that the event has
        for i in 0..(days.num_days() + 1) {

            let current_day = event_date_start.checked_add_days(Days::new(i as u64)).expect("Date overflow");
            if current_day.date() < start_date || current_day.date() > end_date {
                continue;
            }

            // if it is not the first day update the start row
            if i > 0 {
                start_row = 2;
            }
            if i < days.num_days() {
                end_row = 98;
            } else {
                end_row = (end_hour * 4) + (end_minute / 15) + 2;
            }

            let title = event.summary.clone().unwrap_or("".to_string());
            let title_escaped = escape(&title, Html).to_string();

            html.push_str(&format!(
                r#"<div class="week-view-event" style="margin-left: {}%; width: {}%; grid-column: {}; grid-row: {}/{}; z-index: {};">
                    <div class="week-view-event-div" style="--event-bg-r: {}; --event-bg-g: {}; --event-bg-b:{}">
                        <div class="week-view-event-title">{}</div>
                        <div class="week-view-event-date">{}</div>
                        <div class="week-view-event-time">{} - {}</div>
                    </div>
                </div>"#,
                left, (100 - left),
                (event_date_start.weekday().number_from_monday() + i as u32 - 1) % 7 + 2,
                start_row, end_row,
                overlaps_map_index,
                random_hex_color_r,
                random_hex_color_g,
                random_hex_color_b,
                title_escaped,
                format!("{} {}", event_date_start.format("%b"), event_date_start.day()),
                format!("{:02}:{:02}", start_hour, start_minute),
                format!("{:02}:{:02}", end_hour, end_minute),
            ));
        }
    }
    html
}
