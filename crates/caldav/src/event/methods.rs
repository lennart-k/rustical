use crate::CalDavContext;
use crate::Error;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_store::CalendarStore;

pub async fn delete_event<A: CheckAuthentication, C: CalendarStore + ?Sized>(
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
    context.store.write().await.delete_event(&cid, &uid).await?;

    Ok(HttpResponse::Ok().body(""))
}

pub async fn get_event<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String, String)>,
    auth: AuthInfoExtractor<A>,
) -> Result<HttpResponse, Error> {
    // TODO: verify whether user is authorized
    let (principal, cid, mut uid) = path.into_inner();

    if auth.inner.user_id != principal {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    let calendar = context.store.read().await.get_calendar(&cid).await?;
    if auth.inner.user_id != calendar.owner {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    if uid.ends_with(".ics") {
        uid.truncate(uid.len() - 4);
    }
    let event = context.store.read().await.get_event(&cid, &uid).await?;

    Ok(HttpResponse::Ok()
        .insert_header(("ETag", event.get_etag()))
        .body(event.get_ics().to_owned()))
}

pub async fn put_event<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String, String)>,
    body: String,
    auth: AuthInfoExtractor<A>,
) -> Result<HttpResponse, Error> {
    let (principal, cid, mut uid) = path.into_inner();
    let auth_info = auth.inner;
    if auth_info.user_id != principal {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    let calendar = context.store.read().await.get_calendar(&cid).await?;
    if auth_info.user_id != calendar.owner {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    // Incredibly bodged method of normalising the uid but works for a prototype
    if uid.ends_with(".ics") {
        uid.truncate(uid.len() - 4);
    }
    context
        .store
        .write()
        .await
        .upsert_event(cid, uid, body)
        .await?;

    Ok(HttpResponse::Ok().body(""))
}