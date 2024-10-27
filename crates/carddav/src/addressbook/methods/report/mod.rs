use crate::Error;
use actix_web::{
    web::{Data, Path},
    HttpRequest, Responder,
};
use addressbook_multiget::{handle_addressbook_multiget, AddressbookMultigetRequest};
use rustical_store::{auth::User, AddressbookStore};
use serde::{Deserialize, Serialize};
use sync_collection::{handle_sync_collection, SyncCollectionRequest};
use tracing::instrument;

mod addressbook_multiget;
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
    AddressbookMultiget(AddressbookMultigetRequest),
    SyncCollection(SyncCollectionRequest),
}

#[instrument(skip(req, addr_store))]
pub async fn route_report_addressbook<AS: AddressbookStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    req: HttpRequest,
    addr_store: Data<AS>,
) -> Result<impl Responder, Error> {
    let (principal, addressbook_id) = path.into_inner();
    if principal != user.id {
        return Err(Error::Unauthorized);
    }

    let request: ReportRequest = quick_xml::de::from_str(&body)?;

    Ok(match request.clone() {
        ReportRequest::AddressbookMultiget(addr_multiget) => {
            handle_addressbook_multiget(
                addr_multiget,
                req,
                &principal,
                &addressbook_id,
                addr_store.as_ref(),
            )
            .await?
        }
        ReportRequest::SyncCollection(sync_collection) => {
            handle_sync_collection(
                sync_collection,
                req,
                &principal,
                &addressbook_id,
                addr_store.as_ref(),
            )
            .await?
        }
    })
}
