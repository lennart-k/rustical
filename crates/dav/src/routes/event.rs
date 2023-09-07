use crate::{CalDavContext, Error};
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_store::calendar::CalendarStore;

pub async fn delete_event<A: CheckAuthentication, C: CalendarStore>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String, String)>,
    auth: AuthInfoExtractor<A>,
) -> Result<HttpResponse, Error> {
    let _user = auth.inner.user_id;
    // TODO: verify whether user is authorized
    let (_principal, mut cid, uid) = path.into_inner();
    if cid.ends_with(".ics") {
        cid.truncate(cid.len() - 4);
    }
    context
        .store
        .write()
        .await
        .delete_event(&cid, &uid)
        .await
        .map_err(|_e| Error::InternalError)?;

    Ok(HttpResponse::Ok().body(""))
}

pub async fn get_event<A: CheckAuthentication, C: CalendarStore>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String, String)>,
    _auth: AuthInfoExtractor<A>,
) -> Result<HttpResponse, Error> {
    // TODO: verify whether user is authorized
    let (_principal, cid, mut uid) = path.into_inner();
    if uid.ends_with(".ics") {
        uid.truncate(uid.len() - 4);
    }
    let event = context
        .store
        .read()
        .await
        .get_event(&cid, &uid)
        .await
        .map_err(|_e| Error::NotFound)?;

    Ok(HttpResponse::Ok()
        .insert_header(("ETag", event.get_etag()))
        .body(event.to_ics().to_string()))
}

pub async fn put_event<A: CheckAuthentication, C: CalendarStore>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String, String)>,
    body: String,
    _auth: AuthInfoExtractor<A>,
) -> Result<HttpResponse, Error> {
    // TODO: verify whether user is authorized
    let (_principal, cid, mut uid) = path.into_inner();
    // Incredibly bodged method of normalising the uid but works for a prototype
    if uid.ends_with(".ics") {
        uid.truncate(uid.len() - 4);
    }
    dbg!(&body);
    context
        .store
        .write()
        .await
        .upsert_event(cid, uid, body)
        .await
        .map_err(|_e| Error::InternalError)?;

    Ok(HttpResponse::Ok().body(""))
}
