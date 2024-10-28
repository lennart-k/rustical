use crate::Error;
use actix_web::web::Path;
use actix_web::{web::Data, HttpResponse};
use rustical_store::{auth::User, Addressbook, AddressbookStore};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    #[serde(rename = "CARD:addressbook", alias = "addressbook")]
    addressbook: Option<()>,
    collection: Option<()>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct MkcolAddressbookProp {
    resourcetype: Option<Resourcetype>,
    displayname: Option<String>,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PropElement<T: Serialize> {
    prop: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(rename = "mkcol")]
struct MkcolRequest {
    set: PropElement<MkcolAddressbookProp>,
}

pub async fn route_mkcol<AS: AddressbookStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    store: Data<AS>,
) -> Result<HttpResponse, Error> {
    let (principal, addressbook_id) = path.into_inner();
    if principal != user.id {
        return Err(Error::Unauthorized);
    }

    let request: MkcolRequest = quick_xml::de::from_str(&body)?;
    let request = request.set.prop;

    let addressbook = Addressbook {
        id: addressbook_id.to_owned(),
        principal: principal.to_owned(),
        displayname: request.displayname,
        description: request.description,
        deleted_at: None,
        synctoken: 0,
    };

    match store.get_addressbook(&principal, &addressbook_id).await {
        Err(rustical_store::Error::NotFound) => {
            // No conflict, no worries
        }
        Ok(_) => {
            // oh no, there's a conflict
            return Ok(HttpResponse::Conflict().body("An addressbook already exists at this URI"));
        }
        Err(err) => {
            // some other error
            return Err(err.into());
        }
    }

    match store.insert_addressbook(addressbook).await {
        // TODO: The spec says we should return a mkcol-response.
        // However, it works without one but breaks on iPadOS when using an empty one :)
        Ok(()) => Ok(HttpResponse::Created()
            .insert_header(("Cache-Control", "no-cache"))
            .body("")),
        Err(err) => {
            dbg!(err.to_string());
            Err(err.into())
        }
    }
}
