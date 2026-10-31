use crate::Error;
use crate::addressbook::AddressbookResourceService;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::Response;
use axum_extra::headers::{ContentType, HeaderMapExt};
use http::{HeaderValue, Method, StatusCode, header};
use rustical_dav::privileges::UserPrivilege;
use rustical_dav::resource::Resource;
use rustical_dav::rfc_3986_percent_encode;
use rustical_dav_push::DavPushStore;
use rustical_store::AddressbookStore;
use rustical_store::access::{self, Need};
use rustical_store::auth::Principal;
use rustical_store::shared;
use std::str::FromStr;
use tracing::instrument;

#[instrument(skip(service))]
pub async fn route_get<AS: AddressbookStore, DP: DavPushStore>(
    Path((principal, addressbook_id)): Path<(String, String)>,
    State(service): State<AddressbookResourceService<AS, DP>>,
    user: Principal,
    method: Method,
) -> Result<Response, Error> {
    access::check_home(&user, &principal)?;
    let addressbook_resource = service.resource(&principal, &addressbook_id, false).await?;
    addressbook_resource.resolved.check(Need::Read)?;
    if !addressbook_resource
        .get_user_privileges(&user)?
        .has(&UserPrivilege::Read)
    {
        return Err(rustical_store::Error::Forbidden.into());
    }
    let addressbook = addressbook_resource.addressbook;

    let objects =
        shared::address_objects(service.addr_store.as_ref(), &addressbook_resource.resolved)
            .await?;
    let vcf = objects
        .iter()
        .map(|(_id, obj)| obj.get_vcf())
        .collect::<Vec<_>>()
        .join("\r\n");

    let mut resp = Response::builder().status(StatusCode::OK);
    let hdrs = resp.headers_mut().unwrap();
    hdrs.typed_insert(ContentType::from_str("text/vcard; charset=utf-8").unwrap());
    let filename = format!(
        "{principal}_{addressbook_id}{}.vcf",
        addressbook
            .displayname
            .as_deref()
            .map(|name| format!("_{name}"))
            .unwrap_or_default()
    );
    let filename = rfc_3986_percent_encode(&filename);
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
        Ok(resp.body(Body::new(vcf)).unwrap())
    }
}
