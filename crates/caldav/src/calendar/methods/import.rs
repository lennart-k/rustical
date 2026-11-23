use crate::Error;
use crate::calendar::CalendarResourceService;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use caldata::IcalParser;
use http::StatusCode;
use rustical_dav::header::Overwrite;
use rustical_dav_push::DavPushStore;
use rustical_ical::CalendarObjectType;
use rustical_store::{Calendar, CalendarMetadata, CalendarStore, auth::Principal};
use tracing::instrument;

#[instrument(skip(resource_service))]
pub async fn route_import<C: CalendarStore, DP: DavPushStore>(
    Path((principal, cal_id)): Path<(String, String)>,
    user: Principal,
    State(resource_service): State<CalendarResourceService<C, DP>>,
    overwrite: Option<TypedHeader<Overwrite>>,
    body: String,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let overwrite = overwrite
        .map_or_default(|TypedHeader(overwrite)| overwrite)
        .into();

    let parser = IcalParser::from_slice(body.as_bytes());
    let cal = match parser.expect_one() {
        Ok(cal) => cal,
        Err(err) => return Ok((StatusCode::BAD_REQUEST, err.to_string()).into_response()),
    };

    let displayname = cal
        .other_properties
        .iter()
        .find(|prop| prop.name == "X-WR-CALNAME")
        .map(|prop| prop.value.clone());
    let description = cal
        .other_properties
        .iter()
        .find(|prop| prop.name == "X-WR-CALDESC")
        .map(|prop| prop.value.clone());
    let color = cal
        .other_properties
        .iter()
        .find(|prop| prop.name == "X-WR-CALCOLOR")
        .map(|prop| prop.value.clone());
    let timezone_id = cal
        .other_properties
        .iter()
        .find(|prop| prop.name == "X-WR-TIMEZONE")
        .map(|prop| prop.value.clone());

    // Make sure timezone is valid
    if let Some(timezone_id) = timezone_id.as_ref() {
        assert!(
            vtimezones_rs::VTIMEZONES.contains_key(timezone_id),
            "Invalid calendar timezone id"
        );
    }
    // // Extract necessary component types
    let mut cal_components = vec![];
    if !cal.event.is_empty() {
        cal_components.push(CalendarObjectType::Event);
    }
    if !cal.journal.is_empty() {
        cal_components.push(CalendarObjectType::Journal);
    }
    if !cal.todo.is_empty() {
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
