use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension, Form,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::TypedHeader;
use headers::Referer;
use http::StatusCode;
use rustical_ical::CalendarObjectType;
use rustical_store::{Calendar, CalendarStore, auth::User};
use serde::{Deserialize, Deserializer};

#[derive(Template, WebTemplate)]
#[template(path = "pages/calendar.html")]
struct CalendarPage {
    calendar: Calendar,
}

pub async fn route_calendar<C: CalendarStore>(
    Path((owner, cal_id)): Path<(String, String)>,
    Extension(store): Extension<Arc<C>>,
    user: User,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }
    Ok(CalendarPage {
        calendar: store.get_calendar(&owner, &cal_id).await?,
    }
    .into_response())
}

fn empty_to_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let val: Option<String> = Deserialize::deserialize(deserializer)?;
    Ok(val.filter(|val| !val.is_empty()))
}

#[derive(Deserialize, Clone)]
pub struct PutCalendarForm {
    id: String,
    #[serde(deserialize_with = "empty_to_none")]
    displayname: Option<String>,
    #[serde(deserialize_with = "empty_to_none")]
    description: Option<String>,
    #[serde(deserialize_with = "empty_to_none")]
    color: Option<String>,
    #[serde(deserialize_with = "empty_to_none")]
    subscription_url: Option<String>,
    comp_event: Option<String>,
    comp_todo: Option<String>,
    comp_journal: Option<String>,
}

pub async fn route_create_calendar<C: CalendarStore>(
    Path(owner): Path<String>,
    Extension(store): Extension<Arc<C>>,
    user: User,
    Form(PutCalendarForm {
        id,
        displayname,
        description,
        color,
        subscription_url,
        comp_event,
        comp_todo,
        comp_journal,
    }): Form<PutCalendarForm>,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    assert!(!id.is_empty());

    let mut comps = vec![];
    if comp_event.is_some() {
        comps.push(CalendarObjectType::Event);
    }
    if comp_todo.is_some() {
        comps.push(CalendarObjectType::Todo);
    }
    if comp_journal.is_some() {
        comps.push(CalendarObjectType::Journal);
    }

    let cal = Calendar {
        id: id.to_owned(),
        displayname,
        description,
        color,
        subscription_url,
        principal: user.id.to_owned(),
        components: comps,
        order: 0,
        timezone_id: None,
        timezone: None,
        synctoken: 0,
        deleted_at: None,
        push_topic: uuid::Uuid::new_v4().to_string(),
    };

    store.insert_calendar(cal).await?;
    Ok(Redirect::to(&format!("/frontend/user/{}/calendar/{}", user.id, id)).into_response())
}

pub async fn route_calendar_restore<CS: CalendarStore>(
    Path((owner, cal_id)): Path<(String, String)>,
    Extension(store): Extension<Arc<CS>>,
    user: User,
    referer: Option<TypedHeader<Referer>>,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }
    store.restore_calendar(&owner, &cal_id).await?;
    Ok(match referer {
        Some(referer) => Redirect::to(&referer.to_string()).into_response(),
        None => (StatusCode::CREATED, "Restored").into_response(),
    })
}
