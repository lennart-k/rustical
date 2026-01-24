use crate::Error;
use crate::calendar::CalendarResourceService;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use caldata::component::{Component, ComponentMut};
use caldata::{IcalParser, parser::ParserOptions};
use http::StatusCode;
use rustical_dav::header::Overwrite;
use rustical_ical::CalendarObjectType;
use rustical_store::{
    Calendar, CalendarMetadata, CalendarStore, SubscriptionStore, auth::Principal,
};
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

    let parser = IcalParser::from_slice(body.as_bytes());
    let mut cal = match parser.expect_one() {
        Ok(cal) => cal.mutable(),
        Err(err) => return Ok((StatusCode::BAD_REQUEST, err.to_string()).into_response()),
    };

    // Extract calendar metadata
    let displayname = cal
        .get_property("X-WR-CALNAME")
        .and_then(|prop| prop.value.clone());
    let description = cal
        .get_property("X-WR-CALDESC")
        .and_then(|prop| prop.value.clone());
    let color = cal
        .get_property("X-WR-CALCOLOR")
        .and_then(|prop| prop.value.clone());
    let timezone_id = cal
        .get_property("X-WR-TIMEZONE")
        .and_then(|prop| prop.value.clone());
    // These properties should not appear in the expanded calendar objects
    cal.remove_property("X-WR-CALNAME");
    cal.remove_property("X-WR-CALDESC");
    cal.remove_property("X-WR-CALCOLOR");
    cal.remove_property("X-WR-TIMEZONE");
    let cal = cal.build(&ParserOptions::default(), None).unwrap();

    // Make sure timezone is valid
    if let Some(timezone_id) = timezone_id.as_ref() {
        assert!(
            vtimezones_rs::VTIMEZONES.contains_key(timezone_id),
            "Invalid calendar timezone id"
        );
    }
    // // Extract necessary component types
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

    let objects = match cal.into_objects() {
        Ok(objects) => objects.into_iter().map(Into::into).collect(),
        Err(err) => return Ok((StatusCode::BAD_REQUEST, err.to_string()).into_response()),
    };
    let new_cal = Calendar {
        principal,
        id: cal_id,
        meta: CalendarMetadata {
            displayname,
            order: 0,
            description,
            color,
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
