use std::collections::HashMap;

use crate::calendar::prop::SupportedCalendarComponent;
use crate::calendar::{self, CalendarResourceService};
use crate::{Error, calendar_set};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use ical::generator::Emitter;
use ical::parser::ical::component::IcalTimeZone;
use ical::{IcalParser, parser::Component};
use rustical_ical::CalendarObjectType;
use rustical_store::{Calendar, CalendarStore, SubscriptionStore, auth::User};
use tracing::instrument;

#[instrument(skip(cal_store))]
pub async fn route_put<C: CalendarStore, S: SubscriptionStore>(
    Path((principal, cal_id)): Path<(String, String)>,
    State(CalendarResourceService { cal_store, .. }): State<CalendarResourceService<C, S>>,
    user: User,
    body: String,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(crate::Error::Unauthorized);
    }

    let mut parser = IcalParser::new(body.as_bytes());
    let cal = parser
        .next()
        .ok_or(rustical_ical::Error::MissingCalendar)?
        .map_err(rustical_ical::Error::from)?;
    if parser.next().is_some() {
        return Err(rustical_ical::Error::InvalidData(
            "multiple calendars, only one allowed".to_owned(),
        )
        .into());
    }
    if !cal.alarms.is_empty() || !cal.free_busys.is_empty() {
        return Err(rustical_ical::Error::InvalidData(
            "Importer does not support VALARM and VFREEBUSY components".to_owned(),
        )
        .into());
    }

    let mut objects = vec![];
    for event in cal.events {}
    for todo in cal.todos {}
    for journal in cal.journals {}

    let timezones: HashMap<String, IcalTimeZone> = cal
        .timezones
        .clone()
        .into_iter()
        .filter_map(|timezone| {
            let timezone_prop = timezone.get_property("TZID")?.to_owned();
            let tzid = timezone_prop.value?;
            Some((tzid, timezone))
        })
        .collect();

    let displayname = cal.get_property("X-WR-CALNAME").and_then(|prop| prop.value);
    let description = cal.get_property("X-WR-CALDESC").and_then(|prop| prop.value);
    let color = cal
        .get_property("X-RUSTICAL-COLOR")
        .and_then(|prop| prop.value);
    let timezone_id = cal
        .get_property("X-WR-TIMEZONE")
        .and_then(|prop| prop.value);
    let timezone = timezone_id
        .and_then(|tzid| timezones.get(&tzid))
        .map(|timezone| timezone.generate());

    let mut components = vec![CalendarObjectType::Event, CalendarObjectType::Todo];
    if !cal.journals.is_empty() {
        components.push(CalendarObjectType::Journal);
    }

    let calendar = Calendar {
        principal: principal.clone(),
        id: cal_id,
        displayname,
        description,
        color,
        timezone_id,
        timezone,
        components,
        subscription_url: None,
        push_topic: uuid::Uuid::new_v4().to_string(),
        synctoken: 0,
        deleted_at: None,
        order: 0,
    };

    cal_store
        .import_calendar(&principal, calendar, objects)
        .await?;

    Ok(StatusCode::CREATED.into_response())
}
