use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::TypedHeader;
use headers::Referer;
use http::StatusCode;
use rustical_store::{Calendar, CalendarStore, auth::Principal};

#[derive(Template, WebTemplate)]
#[template(path = "pages/calendar.html")]
struct CalendarPage {
    calendar: Calendar,
}

pub async fn route_calendar<C: CalendarStore>(
    Path((owner, cal_id)): Path<(String, String)>,
    Extension(store): Extension<Arc<C>>,
    user: Principal,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }
    Ok(CalendarPage {
        calendar: store.get_calendar(&owner, &cal_id).await?,
    }
    .into_response())
}

pub async fn route_calendar_restore<CS: CalendarStore>(
    Path((owner, cal_id)): Path<(String, String)>,
    Extension(store): Extension<Arc<CS>>,
    user: Principal,
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
