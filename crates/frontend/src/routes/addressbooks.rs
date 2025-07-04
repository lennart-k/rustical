use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::{Extension, extract::Path, response::IntoResponse};
use http::StatusCode;
use rustical_store::{Addressbook, AddressbookStore, CollectionMetadata, auth::Principal};

use crate::pages::user::{Section, UserPage};

impl Section for AddressbooksSection {
    fn name() -> &'static str {
        "addressbooks"
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "components/sections/addressbooks_section.html")]
pub struct AddressbooksSection {
    pub user: Principal,
    pub addressbooks: Vec<(CollectionMetadata, Addressbook)>,
    pub deleted_addressbooks: Vec<(CollectionMetadata, Addressbook)>,
}

pub async fn route_addressbooks<AS: AddressbookStore>(
    Path(user_id): Path<String>,
    Extension(addr_store): Extension<Arc<AS>>,
    user: Principal,
) -> impl IntoResponse {
    if user_id != user.id {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let mut addressbooks = vec![];
    for group in user.memberships() {
        addressbooks.extend(addr_store.get_addressbooks(group).await.unwrap());
    }

    let mut deleted_addressbooks = vec![];
    for group in user.memberships() {
        deleted_addressbooks.extend(addr_store.get_deleted_addressbooks(group).await.unwrap());
    }

    let mut addressbook_infos = vec![];
    for addressbook in addressbooks {
        addressbook_infos.push((
            addr_store
                .addressbook_metadata(&addressbook.principal, &addressbook.id)
                .await
                .unwrap(),
            addressbook,
        ));
    }

    let mut deleted_addressbook_infos = vec![];
    for addressbook in deleted_addressbooks {
        deleted_addressbook_infos.push((
            addr_store
                .addressbook_metadata(&addressbook.principal, &addressbook.id)
                .await
                .unwrap(),
            addressbook,
        ));
    }

    UserPage {
        section: AddressbooksSection {
            user: user.clone(),
            addressbooks: addressbook_infos,
            deleted_addressbooks: deleted_addressbook_infos,
        },
        user,
    }
    .into_response()
}
