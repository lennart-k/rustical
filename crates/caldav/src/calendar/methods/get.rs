use crate::Error;
use crate::calendar::CalendarResourceService;
use axum::body::Body;
use axum::extract::State;
use axum::{extract::Path, response::Response};
use headers::{ContentType, HeaderMapExt};
use http::{HeaderValue, Method, StatusCode, header};
use ical::generator::{Emitter, IcalCalendar, IcalCalendarBuilder};
use ical::property::Property;
use percent_encoding::{CONTROLS, utf8_percent_encode};
use rustical_ical::{CalendarObjectComponent, EventObject};
use rustical_store::calendar_store::CalendarQuery;
use rustical_store::{CalendarStore, SubscriptionStore, auth::Principal};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::instrument;
use chrono::NaiveDate;


// Todo add a generic function to fetch calendar events and data without the need to use the caldav api
pub async fn get_calendar_objects<C: CalendarStore>(
    principal: &str,
    calendar_id: &str,
    start: NaiveDate,
    end: NaiveDate,
    store: &C,
) -> Result<IcalCalendar, Error> {
    let calendar = store.get_calendar(principal, calendar_id, true).await?;

    let mut timezones: HashMap<&String, &Option<chrono_tz::Tz>> = HashMap::new();
    let mut vtimezones: HashMap<&String, &ical::parser::ical::component::IcalTimeZone> = HashMap::new();
    let query = CalendarQuery {
        time_start: Some(start),
        time_end: Some(end),
    };

    let objects = store.calendar_query(&principal, &calendar_id, query).await?;

    let mut ical_calendar_builder = IcalCalendarBuilder::version("4.0")
        .gregorian()
        .prodid("RustiCal");
    if let Some(displayname) = calendar.meta.displayname {
        ical_calendar_builder = ical_calendar_builder.set(Property {
            name: "X-WR-CALNAME".to_owned(),
            value: Some(displayname),
            params: None,
        });
    }
    if let Some(description) = calendar.meta.description {
        ical_calendar_builder = ical_calendar_builder.set(Property {
            name: "X-WR-CALDESC".to_owned(),
            value: Some(description),
            params: None,
        });
    }
    if let Some(timezone_id) = calendar.timezone_id {
        ical_calendar_builder = ical_calendar_builder.set(Property {
            name: "X-WR-TIMEZONE".to_owned(),
            value: Some(timezone_id),
            params: None,
        });
    }

    for object in &objects {
        vtimezones.extend(object.get_vtimezones());
        match object.get_data() {
            CalendarObjectComponent::Event(
                EventObject {
                    event,
                    timezones: object_timezones,
                    ..
                },
                overrides,
            ) => {
                timezones.extend(object_timezones);
                ical_calendar_builder = ical_calendar_builder.add_event(event.clone());
                for _override in overrides {
                    ical_calendar_builder =
                        ical_calendar_builder.add_event(_override.event.clone());
                }
            }
            CalendarObjectComponent::Todo(todo, overrides) => {
                ical_calendar_builder = ical_calendar_builder.add_todo(todo.clone());
                for _override in overrides {
                    ical_calendar_builder = ical_calendar_builder.add_todo(_override.clone());
                }
            }
            CalendarObjectComponent::Journal(journal, overrides) => {
                ical_calendar_builder = ical_calendar_builder.add_journal(journal.clone());
                for _override in overrides {
                    ical_calendar_builder = ical_calendar_builder.add_journal(_override.clone());
                }
            }
        }
    }

    for vtimezone in vtimezones.into_values() {
        ical_calendar_builder = ical_calendar_builder.add_tz(vtimezone.to_owned());
    }

    let ical_calendar = ical_calendar_builder
        .build()
        .map_err(|parser_error| Error::IcalError(parser_error.into()))?;

    Ok(ical_calendar)
}




// Get data when using the CalDAV API
#[instrument(skip(cal_store))]
pub async fn route_get<C: CalendarStore, S: SubscriptionStore>(
    Path((principal, calendar_id)): Path<(String, String)>,
    State(CalendarResourceService { cal_store, .. }): State<CalendarResourceService<C, S>>,
    user: Principal,
    method: Method,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(crate::Error::Unauthorized);
    }

    let calendar = cal_store
        .get_calendar(&principal, &calendar_id, true)
        .await?;
    if !user.is_principal(&calendar.principal) {
        return Err(crate::Error::Unauthorized);
    }

    let mut timezones = HashMap::new();
    let mut vtimezones = HashMap::new();
    let objects = cal_store.get_objects(&principal, &calendar_id).await?;

    let mut ical_calendar_builder = IcalCalendarBuilder::version("4.0")
        .gregorian()
        .prodid("RustiCal");
    if let Some(displayname) = calendar.meta.displayname {
        ical_calendar_builder = ical_calendar_builder.set(Property {
            name: "X-WR-CALNAME".to_owned(),
            value: Some(displayname),
            params: None,
        });
    }
    if let Some(description) = calendar.meta.description {
        ical_calendar_builder = ical_calendar_builder.set(Property {
            name: "X-WR-CALDESC".to_owned(),
            value: Some(description),
            params: None,
        });
    }
    if let Some(timezone_id) = calendar.timezone_id {
        ical_calendar_builder = ical_calendar_builder.set(Property {
            name: "X-WR-TIMEZONE".to_owned(),
            value: Some(timezone_id),
            params: None,
        });
    }

    for object in &objects {
        vtimezones.extend(object.get_vtimezones());
        match object.get_data() {
            CalendarObjectComponent::Event(
                EventObject {
                    event,
                    timezones: object_timezones,
                    ..
                },
                overrides,
            ) => {
                timezones.extend(object_timezones);
                ical_calendar_builder = ical_calendar_builder.add_event(event.clone());
                for _override in overrides {
                    ical_calendar_builder =
                        ical_calendar_builder.add_event(_override.event.clone());
                }
            }
            CalendarObjectComponent::Todo(todo, overrides) => {
                ical_calendar_builder = ical_calendar_builder.add_todo(todo.clone());
                for _override in overrides {
                    ical_calendar_builder = ical_calendar_builder.add_todo(_override.clone());
                }
            }
            CalendarObjectComponent::Journal(journal, overrides) => {
                ical_calendar_builder = ical_calendar_builder.add_journal(journal.clone());
                for _override in overrides {
                    ical_calendar_builder = ical_calendar_builder.add_journal(_override.clone());
                }
            }
        }
    }

    for vtimezone in vtimezones.into_values() {
        ical_calendar_builder = ical_calendar_builder.add_tz(vtimezone.to_owned());
    }

    let ical_calendar = ical_calendar_builder
        .build()
        .map_err(|parser_error| Error::IcalError(parser_error.into()))?;

    let mut resp = Response::builder().status(StatusCode::OK);
    let hdrs = resp.headers_mut().unwrap();
    hdrs.typed_insert(ContentType::from_str("text/calendar").unwrap());

    let filename = format!("{}_{}.ics", calendar.principal, calendar.id);
    let filename = utf8_percent_encode(&filename, CONTROLS);
    hdrs.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachement; filename*=UTF-8''{filename}; filename={filename}",
        ))
        .unwrap(),
    );
    if matches!(method, Method::HEAD) {
        Ok(resp.body(Body::empty()).unwrap())
    } else {
        Ok(resp.body(Body::new(ical_calendar.generate())).unwrap())
    }
}
