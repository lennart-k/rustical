use crate::Error;
use crate::error::Precondition;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::http::header;
use actix_web::http::header::HeaderValue;
use actix_web::web::{Data, Path};
use rustical_ical::CalendarObject;
use rustical_store::CalendarStore;
use rustical_store::auth::User;
use tracing::instrument;
use tracing_actix_web::RootSpan;

use super::resource::CalendarObjectPathComponents;

#[instrument(parent = root_span.id(), skip(store, root_span))]
pub async fn get_event<C: CalendarStore>(
    path: Path<CalendarObjectPathComponents>,
    store: Data<C>,
    user: User,
    root_span: RootSpan,
) -> Result<HttpResponse, Error> {
    let CalendarObjectPathComponents {
        principal,
        calendar_id,
        object_id,
    } = path.into_inner();

    if !user.is_principal(&principal) {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    let calendar = store.get_calendar(&principal, &calendar_id).await?;
    if !user.is_principal(&calendar.principal) {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    let event = store
        .get_object(&principal, &calendar_id, &object_id)
        .await?;

    Ok(HttpResponse::Ok()
        .insert_header(("ETag", event.get_etag()))
        .insert_header(("Content-Type", "text/calendar"))
        .body(event.get_ics().to_owned()))
}

#[instrument(parent = root_span.id(), skip(store, req, root_span))]
pub async fn put_event<C: CalendarStore>(
    path: Path<CalendarObjectPathComponents>,
    store: Data<C>,
    body: String,
    user: User,
    req: HttpRequest,
    root_span: RootSpan,
) -> Result<HttpResponse, Error> {
    let CalendarObjectPathComponents {
        principal,
        calendar_id,
        object_id,
    } = path.into_inner();

    if !user.is_principal(&principal) {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let overwrite =
        Some(&HeaderValue::from_static("*")) != req.headers().get(header::IF_NONE_MATCH);

    let object = match CalendarObject::from_ics(object_id, body) {
        Ok(obj) => obj,
        Err(_) => {
            return Err(Error::PreconditionFailed(Precondition::ValidCalendarData));
        }
    };
    store
        .put_object(principal, calendar_id, object, overwrite)
        .await?;

    Ok(HttpResponse::Created().finish())
}
