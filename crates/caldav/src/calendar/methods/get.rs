use crate::Error;
use crate::calendar::CalendarResourceService;
use axum::body::Body;
use axum::extract::State;
use axum::{extract::Path, response::Response};
use headers::{ContentType, HeaderMapExt};
use http::{HeaderValue, Method, StatusCode, header};
use ical::component::IcalCalendar;
use ical::generator::Emitter;
use ical::property::ContentLine;
use percent_encoding::{CONTROLS, utf8_percent_encode};
use rustical_store::{CalendarStore, SubscriptionStore, auth::Principal};
use std::str::FromStr;
use tracing::instrument;

#[instrument(skip(cal_store))]
pub async fn route_get<C: CalendarStore, S: SubscriptionStore>(
    Path((principal, calendar_id)): Path<(String, String)>,
    State(CalendarResourceService { cal_store, .. }): State<CalendarResourceService<C, S>>,
    user: Principal,
    method: Method,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(crate::Error::Unauthorized);
    }

    let calendar = cal_store
        .get_calendar(&principal, &calendar_id, true)
        .await?;
    if !user.is_principal(&calendar.principal) {
        return Err(crate::Error::Unauthorized);
    }

    let objects = cal_store
        .get_objects(&principal, &calendar_id)
        .await?
        .into_iter()
        .map(|(_, object)| object.into())
        .collect();

    let mut props = vec![];

    if let Some(displayname) = calendar.meta.displayname {
        props.push(ContentLine {
            name: "X-WR-CALNAME".to_owned(),
            value: Some(displayname),
            params: vec![].into(),
        });
    }
    if let Some(description) = calendar.meta.description {
        props.push(ContentLine {
            name: "X-WR-CALDESC".to_owned(),
            value: Some(description),
            params: vec![].into(),
        });
    }
    if let Some(color) = calendar.meta.color {
        props.push(ContentLine {
            name: "X-WR-CALCOLOR".to_owned(),
            value: Some(color),
            params: vec![].into(),
        });
    }
    if let Some(timezone_id) = calendar.timezone_id {
        props.push(ContentLine {
            name: "X-WR-TIMEZONE".to_owned(),
            value: Some(timezone_id),
            params: vec![].into(),
        });
    }

    let export_calendar = IcalCalendar::from_objects("RustiCal Export".to_owned(), objects, props);

    let mut resp = Response::builder().status(StatusCode::OK);
    let hdrs = resp.headers_mut().unwrap();
    hdrs.typed_insert(ContentType::from_str("text/calendar; charset=utf-8").unwrap());

    let filename = format!("{}_{}.ics", calendar.principal, calendar.id);
    let filename = utf8_percent_encode(&filename, CONTROLS);
    hdrs.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachement; filename*=UTF-8''{filename}; filename={filename}",
        ))
        .unwrap(),
    );
    if matches!(method, Method::HEAD) {
        Ok(resp.body(Body::empty()).unwrap())
    } else {
        Ok(resp.body(Body::new(export_calendar.generate())).unwrap())
    }
}
