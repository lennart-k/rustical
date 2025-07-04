use std::sync::Arc;

use crate::pages::user::{Section, UserPage};
use askama::Template;
use askama_web::WebTemplate;
use axum::{Extension, extract::Path, response::IntoResponse};
use http::StatusCode;
use rustical_store::{Calendar, CalendarStore, CollectionMetadata, auth::Principal};

impl Section for CalendarsSection {
    fn name() -> &'static str {
        "calendars"
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "components/sections/calendars_section.html")]
pub struct CalendarsSection {
    pub user: Principal,
    pub calendars: Vec<(CollectionMetadata, Calendar)>,
    pub deleted_calendars: Vec<(CollectionMetadata, Calendar)>,
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

    let mut calendar_infos = vec![];
    for calendar in calendars {
        calendar_infos.push((
            cal_store
                .calendar_metadata(&calendar.principal, &calendar.id)
                .await
                .unwrap(),
            calendar,
        ));
    }

    let mut deleted_calendars = vec![];
    for group in user.memberships() {
        deleted_calendars.extend(cal_store.get_deleted_calendars(group).await.unwrap());
    }

    let mut deleted_calendar_infos = vec![];
    for calendar in deleted_calendars {
        deleted_calendar_infos.push((
            cal_store
                .calendar_metadata(&calendar.principal, &calendar.id)
                .await
                .unwrap(),
            calendar,
        ));
    }

    UserPage {
        section: CalendarsSection {
            user: user.clone(),
            calendars: calendar_infos,
            deleted_calendars: deleted_calendar_infos,
        },
        user,
    }
    .into_response()
}
