use super::AddressObjectPathComponents;
use super::AddressObjectResourceService;
use crate::Error;
use crate::addressbook::resource::AddressbookResource;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum_extra::TypedHeader;
use axum_extra::headers::{ContentType, ETag, HeaderMapExt, IfNoneMatch};
use http::Method;
use http::{HeaderMap, StatusCode};
use rustical_dav::privileges::UserPrivilege;
use rustical_dav::resource::Resource;
use rustical_ical::AddressObject;
use rustical_store::AddressbookStore;
use rustical_store::auth::Principal;
use std::str::FromStr;
use tracing::instrument;

#[instrument(skip(addr_store))]
pub async fn get_object<AS: AddressbookStore>(
    Path(AddressObjectPathComponents {
        principal,
        addressbook_id,
        object_id,
    }): Path<AddressObjectPathComponents>,
    State(AddressObjectResourceService { addr_store }): State<AddressObjectResourceService<AS>>,
    user: Principal,
    method: Method,
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

    let object = addr_store
        .get_object(&principal, &addressbook_id, &object_id, false)
        .await?;

    let mut resp = Response::builder().status(StatusCode::OK);
    let hdrs = resp.headers_mut().unwrap();
    hdrs.typed_insert(ETag::from_str(&object.get_etag()).unwrap());
    hdrs.typed_insert(ContentType::from_str("text/vcard").unwrap());
    if matches!(method, Method::HEAD) {
        Ok(resp.body(Body::empty()).unwrap())
    } else {
        Ok(resp.body(Body::new(object.get_vcf().to_owned())).unwrap())
    }
}

#[instrument(skip(addr_store, body))]
pub async fn put_object<AS: AddressbookStore>(
    Path(AddressObjectPathComponents {
        principal,
        addressbook_id,
        object_id,
    }): Path<AddressObjectPathComponents>,
    State(AddressObjectResourceService { addr_store }): State<AddressObjectResourceService<AS>>,
    user: Principal,
    mut if_none_match: Option<TypedHeader<IfNoneMatch>>,
    header_map: HeaderMap,
    body: String,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
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

    let object = AddressObject::from_vcf(body)?;
    assert_eq!(object.get_id(), object_id);
    addr_store
        .put_object(principal, addressbook_id, object, overwrite)
        .await?;

    Ok(StatusCode::CREATED.into_response())
}
