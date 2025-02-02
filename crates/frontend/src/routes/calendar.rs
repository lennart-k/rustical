use actix_web::{
    http::{header, StatusCode},
    web::{self, Data, Path},
    HttpRequest, HttpResponse, Responder,
};
use askama::Template;
use askama_actix::TemplateToResponse;
use rustical_store::{auth::User, Calendar, CalendarStore};

#[derive(Template)]
#[template(path = "pages/calendar.html")]
struct CalendarPage {
    calendar: Calendar,
}

pub async fn route_calendar<C: CalendarStore>(
    path: Path<(String, String)>,
    store: Data<C>,
    user: User,
) -> Result<impl Responder, rustical_store::Error> {
    let (owner, cal_id) = path.into_inner();
    if !user.is_principal(&owner) {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }
    Ok(CalendarPage {
        calendar: store.get_calendar(&owner, &cal_id).await?,
    }
    .to_response())
}

pub async fn route_calendar_restore<CS: CalendarStore>(
    path: Path<(String, String)>,
    req: HttpRequest,
    store: Data<CS>,
    user: User,
) -> Result<impl Responder, rustical_store::Error> {
    let (owner, cal_id) = path.into_inner();
    if !user.is_principal(&owner) {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }
    store.restore_calendar(&owner, &cal_id).await?;
    Ok(match req.headers().get(header::REFERER) {
        Some(referer) => web::Redirect::to(referer.to_str().unwrap().to_owned())
            .using_status_code(StatusCode::FOUND)
            .respond_to(&req)
            .map_into_boxed_body(),
        None => HttpResponse::Ok().body("Restored"),
    })
}
