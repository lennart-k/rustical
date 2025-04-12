use actix_web::{
    HttpRequest, HttpResponse, Responder,
    http::{StatusCode, header},
    web::{self, Data, Path},
};
use askama::Template;
use askama_web::WebTemplate;
use rustical_store::{Addressbook, AddressbookStore, auth::User};

#[derive(Template, WebTemplate)]
#[template(path = "pages/addressbook.html")]
struct AddressbookPage {
    addressbook: Addressbook,
}

pub async fn route_addressbook<AS: AddressbookStore>(
    path: Path<(String, String)>,
    store: Data<AS>,
    user: User,
    req: HttpRequest,
) -> Result<impl Responder, rustical_store::Error> {
    let (owner, addrbook_id) = path.into_inner();
    if !user.is_principal(&owner) {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }
    Ok(AddressbookPage {
        addressbook: store.get_addressbook(&owner, &addrbook_id).await?,
    }
    .respond_to(&req))
}

pub async fn route_addressbook_restore<AS: AddressbookStore>(
    path: Path<(String, String)>,
    req: HttpRequest,
    store: Data<AS>,
    user: User,
) -> Result<impl Responder, rustical_store::Error> {
    let (owner, addressbook_id) = path.into_inner();
    if !user.is_principal(&owner) {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }
    store.restore_addressbook(&owner, &addressbook_id).await?;
    Ok(match req.headers().get(header::REFERER) {
        Some(referer) => web::Redirect::to(referer.to_str().unwrap().to_owned())
            .using_status_code(StatusCode::FOUND)
            .respond_to(&req)
            .map_into_boxed_body(),
        None => HttpResponse::Ok().body("Restored"),
    })
}
