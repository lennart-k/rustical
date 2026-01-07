use crate::Error;
use crate::calendar::CalendarResourceService;
use axum::body::Body;
use axum::extract::State;
use axum::{extract::Path, response::Response};
use headers::{ContentType, HeaderMapExt};
use http::{HeaderValue, Method, StatusCode, header};
use ical::generator::Emitter;
use ical::property::ContentLine;
use percent_encoding::{CONTROLS, utf8_percent_encode};
use rustical_store::{CalendarStore, SubscriptionStore, auth::Principal};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::instrument;

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

    // let mut vtimezones = HashMap::new();
    // let objects = cal_store.get_objects(&principal, &calendar_id).await?;

    todo!()

    // let mut ical_calendar_builder = IcalCalendarBuilder::version("2.0")
    //     .gregorian()
    //     .prodid("RustiCal");
    // if let Some(displayname) = calendar.meta.displayname {
    //     ical_calendar_builder = ical_calendar_builder.set(ContentLine {
    //         name: "X-WR-CALNAME".to_owned(),
    //         value: Some(displayname),
    //         params: vec![].into(),
    //     });
    // }
    // if let Some(description) = calendar.meta.description {
    //     ical_calendar_builder = ical_calendar_builder.set(ContentLine {
    //         name: "X-WR-CALDESC".to_owned(),
    //         value: Some(description),
    //         params: vec![].into(),
    //     });
    // }
    // if let Some(timezone_id) = calendar.timezone_id {
    //     ical_calendar_builder = ical_calendar_builder.set(ContentLine {
    //         name: "X-WR-TIMEZONE".to_owned(),
    //         value: Some(timezone_id),
    //         params: vec![].into(),
    //     });
    // }
    //
    // for object in &objects {
    //     vtimezones.extend(object.get_vtimezones());
    //     match object.get_data() {
    //         CalendarObjectComponent::Event(EventObject { event, .. }, overrides) => {
    //             ical_calendar_builder = ical_calendar_builder
    //                 .add_event(event.clone())
    //                 .add_events(overrides.iter().map(|ev| ev.event.clone()));
    //         }
    //         CalendarObjectComponent::Todo(todo, overrides) => {
    //             ical_calendar_builder = ical_calendar_builder
    //                 .add_todo(todo.clone())
    //                 .add_todos(overrides.iter().cloned());
    //         }
    //         CalendarObjectComponent::Journal(journal, overrides) => {
    //             ical_calendar_builder = ical_calendar_builder
    //                 .add_journal(journal.clone())
    //                 .add_journals(overrides.iter().cloned());
    //         }
    //     }
    // }
    //
    // ical_calendar_builder = ical_calendar_builder.add_timezones(vtimezones.into_values().cloned());
    //
    // let ical_calendar = ical_calendar_builder
    //     .build()
    //     .map_err(|parser_error| Error::IcalError(parser_error.into()))?;
    //
    // let mut resp = Response::builder().status(StatusCode::OK);
    // let hdrs = resp.headers_mut().unwrap();
    // hdrs.typed_insert(ContentType::from_str("text/calendar; charset=utf-8").unwrap());
    //
    // let filename = format!("{}_{}.ics", calendar.principal, calendar.id);
    // let filename = utf8_percent_encode(&filename, CONTROLS);
    // hdrs.insert(
    //     header::CONTENT_DISPOSITION,
    //     HeaderValue::from_str(&format!(
    //         "attachement; filename*=UTF-8''{filename}; filename={filename}",
    //     ))
    //     .unwrap(),
    // );
    // if matches!(method, Method::HEAD) {
    //     Ok(resp.body(Body::empty()).unwrap())
    // } else {
    //     Ok(resp.body(Body::new(ical_calendar.generate())).unwrap())
    // }
}
