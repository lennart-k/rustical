use crate::Error;
use crate::calendar_object::{CalendarObjectPathComponents, CalendarObjectResourceService};
use crate::error::Precondition;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum_extra::TypedHeader;
use headers::{ContentType, ETag, HeaderMapExt, IfNoneMatch};
use http::{HeaderMap, Method, StatusCode};
use rustical_ical::CalendarObject;
use rustical_store::CalendarStore;
use rustical_store::auth::Principal;
use std::str::FromStr;
use tracing::{debug, error, instrument};

#[instrument(skip(cal_store))]
pub async fn get_event<C: CalendarStore>(
    Path(CalendarObjectPathComponents {
        principal,
        calendar_id,
        object_id,
    }): Path<CalendarObjectPathComponents>,
    State(CalendarObjectResourceService { cal_store }): State<CalendarObjectResourceService<C>>,
    user: Principal,
    method: Method,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(crate::Error::Unauthorized);
    }

    let calendar = cal_store
        .get_calendar(&principal, &calendar_id, false)
        .await?;
    if !user.is_principal(&calendar.principal) {
        return Err(crate::Error::Unauthorized);
    }

    let event = cal_store
        .get_object(&principal, &calendar_id, &object_id, false)
        .await?;

    let mut resp = Response::builder().status(StatusCode::OK);
    let hdrs = resp.headers_mut().unwrap();
    hdrs.typed_insert(ETag::from_str(&event.get_etag()).unwrap());
    hdrs.typed_insert(ContentType::from_str("text/calendar").unwrap());
    if matches!(method, Method::HEAD) {
        Ok(resp.body(Body::empty()).unwrap())
    } else {
        Ok(resp.body(Body::new(event.get_ics().to_owned())).unwrap())
    }
}

#[instrument(skip(cal_store))]
pub async fn put_event<C: CalendarStore>(
    Path(CalendarObjectPathComponents {
        principal,
        calendar_id,
        object_id,
    }): Path<CalendarObjectPathComponents>,
    State(CalendarObjectResourceService { cal_store }): State<CalendarObjectResourceService<C>>,
    user: Principal,
    mut if_none_match: Option<TypedHeader<IfNoneMatch>>,
    header_map: HeaderMap,
    body: String,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(crate::Error::Unauthorized);
    }

    // https://github.com/hyperium/headers/issues/204
    if !header_map.contains_key("If-None-Match") {
        if_none_match = None;
    }

    let overwrite = if let Some(TypedHeader(if_none_match)) = if_none_match {
        if_none_match == IfNoneMatch::any()
    } else {
        true
    };

    let object = match CalendarObject::from_ics(body.clone()) {
        Ok(obj) => obj,
        Err(_) => {
            debug!("invalid calendar data:\n{body}");
            return Err(Error::PreconditionFailed(Precondition::ValidCalendarData));
        }
    };
    if object.get_id() != object_id {
        error!(
            "Calendar object UID and file name not matching: UID={}, filename={}",
            object.get_id(),
            object_id
        );
        return Err(Error::PreconditionFailed(Precondition::MatchingUid));
    }
    cal_store
        .put_object(principal, calendar_id, object, overwrite)
        .await?;

    Ok(StatusCode::CREATED.into_response())
}
