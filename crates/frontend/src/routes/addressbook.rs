use actix_web::{
    http::{header, StatusCode},
    web::{self, Data, Path},
    HttpRequest, HttpResponse, Responder,
};
use askama::Template;
use rustical_store::{auth::User, Addressbook, AddressbookStore};

#[derive(Template)]
#[template(path = "pages/addressbook.html")]
struct AddressbookPage {
    addressbook: Addressbook,
}

pub async fn route_addressbook<AS: AddressbookStore>(
    path: Path<(String, String)>,
    store: Data<AS>,
    _user: User,
) -> Result<impl Responder, rustical_store::Error> {
    let (owner, addrbook_id) = path.into_inner();
    Ok(AddressbookPage {
        addressbook: store.get_addressbook(&owner, &addrbook_id).await?,
    })
}

pub async fn route_addressbook_restore<AS: AddressbookStore>(
    path: Path<(String, String)>,
    req: HttpRequest,
    store: Data<AS>,
    _user: User,
) -> Result<impl Responder, rustical_store::Error> {
    let (owner, addressbook_id) = path.into_inner();
    store.restore_addressbook(&owner, &addressbook_id).await?;
    Ok(match req.headers().get(header::REFERER) {
        Some(referer) => web::Redirect::to(referer.to_str().unwrap().to_owned())
            .using_status_code(StatusCode::FOUND)
            .respond_to(&req)
            .map_into_boxed_body(),
        None => HttpResponse::Ok().body("Restored"),
    })
}
