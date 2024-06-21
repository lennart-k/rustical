use crate::CalDavContext;
use crate::Error;
use actix_web::HttpRequest;
use actix_web::{
    web::{Data, Path},
    HttpResponse,
};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_store::CalendarStore;

pub async fn route_delete_calendar<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String)>,
    auth: AuthInfoExtractor<A>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (principal, cid) = path.into_inner();
    if principal != auth.inner.user_id {
        return Err(Error::Unauthorized);
    }

    let no_trash = req
        .headers()
        .get("X-No-Trashbin")
        .map(|val| matches!(val.to_str(), Ok("1")))
        .unwrap_or(false);

    context
        .store
        .write()
        .await
        .delete_calendar(&principal, &cid, !no_trash)
        .await?;

    Ok(HttpResponse::Ok().body(""))
}
