use crate::Error;
use actix_web::{
    web::{Data, Path},
    HttpRequest, Responder,
};
use calendar_multiget::{handle_calendar_multiget, CalendarMultigetRequest};
use calendar_query::{handle_calendar_query, CalendarQueryRequest};
use rustical_store::{auth::User, CalendarStore};
use serde::{Deserialize, Serialize};
use sync_collection::{handle_sync_collection, SyncCollectionRequest};
use tracing::instrument;

mod calendar_multiget;
mod calendar_query;
mod sync_collection;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PropQuery {
    Allprop,
    Prop,
    Propname,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ReportRequest {
    CalendarMultiget(CalendarMultigetRequest),
    CalendarQuery(CalendarQueryRequest),
    SyncCollection(SyncCollectionRequest),
}

#[instrument(skip(req, cal_store))]
pub async fn route_report_calendar<C: CalendarStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    req: HttpRequest,
    cal_store: Data<C>,
) -> Result<impl Responder, Error> {
    let (principal, cal_id) = path.into_inner();
    if principal != user.id {
        return Err(Error::Unauthorized);
    }

    let request: ReportRequest = quick_xml::de::from_str(&body)?;

    Ok(match request.clone() {
        ReportRequest::CalendarQuery(cal_query) => {
            handle_calendar_query(
                cal_query,
                req,
                &user,
                &principal,
                &cal_id,
                cal_store.as_ref(),
            )
            .await?
        }
        ReportRequest::CalendarMultiget(cal_multiget) => {
            handle_calendar_multiget(
                cal_multiget,
                req,
                &user,
                &principal,
                &cal_id,
                cal_store.as_ref(),
            )
            .await?
        }
        ReportRequest::SyncCollection(sync_collection) => {
            handle_sync_collection(
                sync_collection,
                req,
                &user,
                &principal,
                &cal_id,
                cal_store.as_ref(),
            )
            .await?
        }
    })
}
