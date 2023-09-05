use crate::{CalDavContext, Error};
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use rustical_store::calendar::CalendarStore;

pub async fn delete_event<C: CalendarStore>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String, String)>,
) -> Result<HttpResponse, Error> {
    let (_principal, mut cid, uid) = path.into_inner();
    if cid.ends_with(".ics") {
        cid.truncate(cid.len() - 4);
    }
    context
        .store
        .write()
        .await
        .delete_event(&uid)
        .await
        .map_err(|_e| Error::InternalError)?;

    Ok(HttpResponse::Ok().body(""))
}

pub async fn get_event<C: CalendarStore>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String, String)>,
) -> Result<HttpResponse, Error> {
    let (_principal, mut cid, uid) = path.into_inner();
    if cid.ends_with(".ics") {
        cid.truncate(cid.len() - 4);
    }
    let event = context
        .store
        .read()
        .await
        .get_event(&uid)
        .await
        .map_err(|_e| Error::NotFound)?;

    Ok(HttpResponse::Ok()
        .insert_header(("ETag", event.get_etag()))
        .body(event.to_ics().to_string()))
}

pub async fn put_event<C: CalendarStore>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String, String)>,
    body: String,
) -> Result<HttpResponse, Error> {
    let (_principal, mut cid, uid) = path.into_inner();
    // Incredibly bodged method of normalising the uid but works for a prototype
    if cid.ends_with(".ics") {
        cid.truncate(cid.len() - 4);
    }
    dbg!(&body);
    context
        .store
        .write()
        .await
        .upsert_event(uid, body)
        .await
        .map_err(|_e| Error::InternalError)?;

    Ok(HttpResponse::Ok().body(""))
}
