use crate::Error;
use crate::addressbook::AddressbookResourceService;
use crate::addressbook::resource::AddressbookResource;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::Response;
use axum_extra::headers::{ContentType, HeaderMapExt};
use http::{HeaderValue, StatusCode, header};
use percent_encoding::{CONTROLS, utf8_percent_encode};
use rustical_dav::privileges::UserPrivilege;
use rustical_dav::resource::Resource;
use rustical_ical::AddressObject;
use rustical_store::auth::User;
use rustical_store::{AddressbookStore, SubscriptionStore};
use std::str::FromStr;
use tracing::instrument;

#[instrument(skip(addr_store))]
pub async fn route_get<AS: AddressbookStore, S: SubscriptionStore>(
    Path((principal, addressbook_id)): Path<(String, String)>,
    State(AddressbookResourceService { addr_store, .. }): State<AddressbookResourceService<AS, S>>,
    user: User,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let addressbook = addr_store
        .get_addressbook(&principal, &addressbook_id, false)
        .await?;
    let addressbook_resource = AddressbookResource(addressbook);
    if !addressbook_resource
        .get_user_privileges(&user)?
        .has(&UserPrivilege::Read)
    {
        return Err(Error::Unauthorized);
    }

    let objects = addr_store.get_objects(&principal, &addressbook_id).await?;
    let vcf = objects
        .iter()
        .map(AddressObject::get_vcf)
        .collect::<Vec<_>>()
        .join("\r\n");

    let mut resp = Response::builder().status(StatusCode::OK);
    let hdrs = resp.headers_mut().unwrap();
    hdrs.typed_insert(ContentType::from_str("text/vcard").unwrap());
    let filename = format!("{}_{}.vcf", principal, addressbook_id);
    let filename = utf8_percent_encode(&filename, CONTROLS);
    hdrs.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachement; filename*=UTF-8''{filename}; filename={filename}",
        ))
        .unwrap(),
    );
    Ok(resp.body(Body::new(vcf)).unwrap())
}
