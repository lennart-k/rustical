use std::sync::Arc;

use crate::pages::user::{Section, UserPage};
use askama::Template;
use askama_web::WebTemplate;
use axum::{Extension, extract::Path, response::IntoResponse};
use http::StatusCode;
use rustical_store::{Calendar, CalendarStore, auth::Principal};

impl Section for CalendarsSection {
    fn name() -> &'static str {
        "calendars"
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "components/sections/calendars_section.html")]
pub struct CalendarsSection {
    pub user: Principal,
    pub calendars: Vec<Calendar>,
    pub deleted_calendars: Vec<Calendar>,
}

pub async fn route_calendars<CS: CalendarStore>(
    Path(user_id): Path<String>,
    Extension(cal_store): Extension<Arc<CS>>,
    user: Principal,
) -> impl IntoResponse {
    if user_id != user.id {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let mut calendars = vec![];
    for group in user.memberships() {
        calendars.extend(cal_store.get_calendars(group).await.unwrap());
    }

    let mut deleted_calendars = vec![];
    for group in user.memberships() {
        deleted_calendars.extend(cal_store.get_deleted_calendars(group).await.unwrap());
    }

    UserPage {
        section: CalendarsSection {
            user: user.clone(),
            calendars,
            deleted_calendars,
        },
        user,
    }
    .into_response()
}
