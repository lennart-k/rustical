use crate::Error;
use actix_web::{
    web::{Data, Path},
    HttpRequest, Responder,
};
use calendar_multiget::{handle_calendar_multiget, CalendarMultigetRequest};
use calendar_query::{handle_calendar_query, CalendarQueryRequest};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_dav::methods::propfind::ServicePrefix;
use rustical_store::CalendarStore;
use serde::{Deserialize, Serialize};
use sync_collection::{handle_sync_collection, SyncCollectionRequest};
use tokio::sync::RwLock;

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

pub async fn route_report_calendar<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    auth: AuthInfoExtractor<A>,
    req: HttpRequest,
    cal_store: Data<RwLock<C>>,
    prefix: Data<ServicePrefix>,
) -> Result<impl Responder, Error> {
    let (principal, cid) = path.into_inner();
    if principal != auth.inner.user_id {
        return Err(Error::Unauthorized);
    }

    dbg!("REPORT request:", &body);
    dbg!(req.headers().get("If"));
    let request: ReportRequest = quick_xml::de::from_str(&body)?;

    Ok(match request.clone() {
        ReportRequest::CalendarQuery(cal_query) => {
            handle_calendar_query(cal_query, req, &prefix.0, &principal, &cid, &cal_store).await?
        }
        ReportRequest::CalendarMultiget(cal_multiget) => {
            handle_calendar_multiget(cal_multiget, req, &prefix.0, &principal, &cid, &cal_store)
                .await?
        }
        ReportRequest::SyncCollection(sync_collection) => {
            handle_sync_collection(
                sync_collection,
                req,
                &prefix.0,
                &principal,
                &cid,
                &cal_store,
            )
            .await?
        }
    })
}
