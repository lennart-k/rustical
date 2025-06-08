use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension, Form,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::TypedHeader;
use headers::Referer;
use http::StatusCode;
use rustical_store::{Addressbook, AddressbookStore, auth::User};
use serde::{Deserialize, Deserializer};

#[derive(Template, WebTemplate)]
#[template(path = "pages/addressbook.html")]
struct AddressbookPage {
    addressbook: Addressbook,
}

pub async fn route_addressbook<AS: AddressbookStore>(
    Path((owner, addrbook_id)): Path<(String, String)>,
    Extension(store): Extension<Arc<AS>>,
    user: User,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }
    Ok(AddressbookPage {
        addressbook: store.get_addressbook(&owner, &addrbook_id, true).await?,
    }
    .into_response())
}

fn empty_to_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let val: Option<String> = Deserialize::deserialize(deserializer)?;
    Ok(val.filter(|val| !val.is_empty()))
}

#[derive(Deserialize, Clone)]
pub struct PutAddressbookForm {
    id: String,
    #[serde(deserialize_with = "empty_to_none")]
    displayname: Option<String>,
    #[serde(deserialize_with = "empty_to_none")]
    description: Option<String>,
}

pub async fn route_create_addressbook<AS: AddressbookStore>(
    Path(owner): Path<String>,
    Extension(store): Extension<Arc<AS>>,
    user: User,
    Form(PutAddressbookForm {
        id,
        displayname,
        description,
    }): Form<PutAddressbookForm>,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    assert!(!id.is_empty());

    let addressbook = Addressbook {
        id: id.to_owned(),
        displayname,
        description,
        principal: user.id.to_owned(),
        synctoken: 0,
        deleted_at: None,
        push_topic: uuid::Uuid::new_v4().to_string(),
    };

    store.insert_addressbook(addressbook).await?;
    Ok(Redirect::to(&format!("/frontend/user/{}/addressbook/{}", user.id, id)).into_response())
}

pub async fn route_addressbook_restore<AS: AddressbookStore>(
    Path((owner, addressbook_id)): Path<(String, String)>,
    Extension(store): Extension<Arc<AS>>,
    user: User,
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
    user: User,
) -> Result<Response, rustical_store::Error> {
    if !user.is_principal(&owner) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    store
        .delete_addressbook(&owner, &addressbook_id, true)
        .await?;

    Ok(Redirect::to(&format!("/frontend/user/{}", user.id)).into_response())
}
