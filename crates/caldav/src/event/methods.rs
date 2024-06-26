use crate::CalDavContext;
use crate::Error;
use actix_web::http::header;
use actix_web::http::header::HeaderValue;
use actix_web::web::{Data, Path};
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_store::CalendarStore;

pub async fn get_event<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String, String)>,
    auth: AuthInfoExtractor<A>,
) -> Result<HttpResponse, Error> {
    let (principal, cid, mut uid) = path.into_inner();

    if auth.inner.user_id != principal {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    let calendar = context
        .store
        .read()
        .await
        .get_calendar(&principal, &cid)
        .await?;
    if auth.inner.user_id != calendar.principal {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    if uid.ends_with(".ics") {
        uid.truncate(uid.len() - 4);
    }
    let event = context
        .store
        .read()
        .await
        .get_event(&principal, &cid, &uid)
        .await?;

    Ok(HttpResponse::Ok()
        .insert_header(("ETag", event.get_etag()))
        .insert_header(("Content-Type", "text/calendar"))
        .body(event.get_ics().to_owned()))
}

pub async fn put_event<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String, String)>,
    body: String,
    auth: AuthInfoExtractor<A>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (principal, cid, mut uid) = path.into_inner();
    let auth_info = auth.inner;
    if auth_info.user_id != principal {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    let calendar = context
        .store
        .read()
        .await
        .get_calendar(&principal, &cid)
        .await?;
    if auth_info.user_id != calendar.principal {
        return Ok(HttpResponse::Unauthorized().body(""));
    }
    // Incredibly bodged method of normalising the uid but works for a prototype
    if uid.ends_with(".ics") {
        uid.truncate(uid.len() - 4);
    }

    // TODO: implement If-Match

    // Lock the store
    let mut store = context.store.write().await;

    if Some(&HeaderValue::from_static("*")) == req.headers().get(header::IF_NONE_MATCH) {
        // Only write if not existing
        match store.get_event(&principal, &cid, &uid).await {
            Ok(_) => {
                // Conflict
                return Ok(HttpResponse::Conflict().body("Resource with this URI already existing"));
            }
            Err(rustical_store::Error::NotFound) => {
                // Path unused, we can proceed
            }
            Err(err) => {
                // Some unknown error :(
                return Err(err.into());
            }
        }
    }

    store.put_event(principal, cid, uid, body).await?;

    Ok(HttpResponse::Created().body(""))
}
