use crate::Error;
use crate::addressbook::AddressbookResourceService;
use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    response::Response,
};
use http::StatusCode;
use ical::VcardParser;
use rustical_ical::AddressObject;
use rustical_store::Addressbook;
use rustical_store::{AddressbookStore, SubscriptionStore, auth::User};
use tracing::instrument;

#[instrument(skip(addr_store))]
pub async fn route_put<AS: AddressbookStore, S: SubscriptionStore>(
    Path((principal, addressbook_id)): Path<(String, String)>,
    State(AddressbookResourceService { addr_store, .. }): State<AddressbookResourceService<AS, S>>,
    user: User,
    body: String,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let mut objects = vec![];
    for object in VcardParser::new(body.as_bytes()) {
        let object = object.map_err(rustical_ical::Error::from)?;
        objects.push(AddressObject::try_from(object)?);
    }

    let addressbook = Addressbook {
        id: addressbook_id.clone(),
        principal: principal.clone(),
        displayname: None,
        description: None,
        deleted_at: None,
        synctoken: Default::default(),
        push_topic: uuid::Uuid::new_v4().to_string(),
    };

    addr_store
        .import_addressbook(principal.clone(), addressbook, objects)
        .await?;

    Ok(StatusCode::CREATED.into_response())
}
