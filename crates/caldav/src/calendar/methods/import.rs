use crate::Error;
use crate::calendar::CalendarResourceService;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use ical::{
    generator::Emitter,
    parser::{Component, ComponentMut},
};
use rustical_dav::header::Overwrite;
use rustical_ical::{CalendarObject, CalendarObjectType};
use rustical_store::{
    Calendar, CalendarMetadata, CalendarStore, SubscriptionStore, auth::Principal,
};
use std::io::BufReader;
use tracing::instrument;

#[instrument(skip(resource_service))]
pub async fn route_import<C: CalendarStore, S: SubscriptionStore>(
    Path((principal, cal_id)): Path<(String, String)>,
    user: Principal,
    State(resource_service): State<CalendarResourceService<C, S>>,
    Overwrite(overwrite): Overwrite,
    body: String,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let mut parser = ical::IcalParser::new(BufReader::new(body.as_bytes()));
    let mut cal = parser
        .next()
        .expect("input must contain calendar")
        .unwrap()
        .mutable();
    if parser.next().is_some() {
        return Err(rustical_ical::Error::InvalidData(
            "multiple calendars, only one allowed".to_owned(),
        )
        .into());
    }

    // Extract calendar metadata
    let displayname = cal
        .get_property("X-WR-CALNAME")
        .and_then(|prop| prop.value.to_owned());
    let description = cal
        .get_property("X-WR-CALDESC")
        .and_then(|prop| prop.value.to_owned());
    let timezone_id = cal
        .get_property("X-WR-TIMEZONE")
        .and_then(|prop| prop.value.to_owned());
    // These properties should not appear in the expanded calendar objects
    cal.remove_property("X-WR-CALNAME");
    cal.remove_property("X-WR-CALDESC");
    cal.remove_property("X-WR-TIMEZONE");
    let cal = cal.verify().unwrap();
    // Make sure timezone is valid
    if let Some(timezone_id) = timezone_id.as_ref() {
        assert!(
            vtimezones_rs::VTIMEZONES.contains_key(timezone_id),
            "Invalid calendar timezone id"
        );
    }

    // Extract necessary component types
    let mut cal_components = vec![];
    if !cal.events.is_empty() {
        cal_components.push(CalendarObjectType::Event);
    }
    if !cal.journals.is_empty() {
        cal_components.push(CalendarObjectType::Journal);
    }
    if !cal.todos.is_empty() {
        cal_components.push(CalendarObjectType::Todo);
    }

    let expanded_cals = cal.expand_calendar();
    // Janky way to convert between IcalCalendar and CalendarObject
    let objects = expanded_cals
        .into_iter()
        .map(|cal| cal.generate())
        .map(CalendarObject::from_ics)
        .collect::<Result<Vec<_>, _>>()?;
    let new_cal = Calendar {
        principal,
        id: cal_id,
        meta: CalendarMetadata {
            displayname,
            order: 0,
            description,
            color: None,
        },
        timezone_id,
        deleted_at: None,
        synctoken: 0,
        subscription_url: None,
        push_topic: uuid::Uuid::new_v4().to_string(),
        components: cal_components,
    };

    let cal_store = resource_service.cal_store;
    cal_store
        .import_calendar(new_cal, objects, overwrite)
        .await?;

    Ok(StatusCode::OK.into_response())
}
