use crate::CalDavContext;
use crate::Error;
use actix_web::{
    web::{Data, Path},
    HttpResponse,
};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_store::calendar::CalendarStore;

pub async fn route_delete_calendar<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String)>,
    auth: AuthInfoExtractor<A>,
) -> Result<HttpResponse, Error> {
    let (principal, cid) = path.into_inner();
    if principal != auth.inner.user_id {
        return Err(Error::Unauthorized);
    }
    context.store.write().await.delete_calendar(&cid).await?;

    Ok(HttpResponse::Ok().body(""))
}
