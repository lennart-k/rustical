use std::{collections::HashMap, io::BufReader};

use crate::Error;
use crate::addressbook::AddressbookResourceService;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use ical::{
    parser::{Component, ComponentMut, vcard},
    property::ContentLine,
};
use rustical_store::{Addressbook, AddressbookStore, SubscriptionStore, auth::Principal};
use tracing::instrument;

#[instrument(skip(resource_service))]
pub async fn route_import<AS: AddressbookStore, S: SubscriptionStore>(
    Path((principal, addressbook_id)): Path<(String, String)>,
    user: Principal,
    State(resource_service): State<AddressbookResourceService<AS, S>>,
    body: String,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let parser = vcard::VcardParser::new(BufReader::new(body.as_bytes()));

    let mut objects = vec![];
    for res in parser {
        let mut card = res.unwrap();
        let uid = card.get_uid();
        if uid.is_none() {
            let mut card_mut = card.mutable();
            card_mut.add_content_line(ContentLine {
                name: "UID".to_owned(),
                value: Some(uuid::Uuid::new_v4().to_string()),
                params: vec![].into(),
            });
            card = card_mut.build(&HashMap::new()).unwrap();
        }

        objects.push(card.try_into().unwrap());
    }

    if objects.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "empty addressbook data").into_response());
    }

    let addressbook = Addressbook {
        principal,
        id: addressbook_id,
        displayname: None,
        description: None,
        deleted_at: None,
        synctoken: 0,
        push_topic: uuid::Uuid::new_v4().to_string(),
    };

    let addr_store = resource_service.addr_store;
    addr_store
        .import_addressbook(addressbook, objects, false)
        .await?;

    Ok(StatusCode::OK.into_response())
}
