use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::TypedHeader;
use headers::Referer;
use http::StatusCode;
use rustical_store::{Addressbook, AddressbookStore, auth::Principal};

#[derive(Template, WebTemplate)]
#[template(path = "pages/addressbook.html")]
struct AddressbookPage {
    addressbook: Addressbook,
}

pub async fn route_addressbook<AS: AddressbookStore>(
    Path((owner, addrbook_id)): Path<(String, String)>,
    Extension(store): Extension<Arc<AS>>,
    user: Principal,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }
    Ok(AddressbookPage {
        addressbook: store.get_addressbook(&owner, &addrbook_id, true).await?,
    }
    .into_response())
}

pub async fn route_addressbook_restore<AS: AddressbookStore>(
    Path((owner, addressbook_id)): Path<(String, String)>,
    Extension(store): Extension<Arc<AS>>,
    user: Principal,
    referer: Option<TypedHeader<Referer>>,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }
    store.restore_addressbook(&owner, &addressbook_id).await?;
    Ok(match referer {
        Some(referer) => Redirect::to(&referer.to_string()).into_response(),
        None => (StatusCode::CREATED, "Restored").into_response(),
    })
}

pub async fn route_delete_addressbook<AS: AddressbookStore>(
    Path((owner, addressbook_id)): Path<(String, String)>,
    Extension(store): Extension<Arc<AS>>,
    user: Principal,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    store
        .delete_addressbook(&owner, &addressbook_id, true)
        .await?;

    Ok(Redirect::to(&format!("/frontend/user/{}", user.id)).into_response())
}
