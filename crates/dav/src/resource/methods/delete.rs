use crate::Error;
use crate::privileges::UserPrivilege;
use crate::resource::Resource;
use crate::resource::ResourceService;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::http::header::IfMatch;
use actix_web::http::header::IfNoneMatch;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Path;
use tracing::instrument;
use tracing_actix_web::RootSpan;

#[instrument(parent = root_span.id(), skip(path, req, root_span, resource_service))]
pub async fn route_delete<R: ResourceService>(
    path: Path<R::PathComponents>,
    req: HttpRequest,
    principal: R::Principal,
    resource_service: Data<R>,
    root_span: RootSpan,
    if_match: web::Header<IfMatch>,
    if_none_match: web::Header<IfNoneMatch>,
) -> Result<impl Responder, R::Error> {
    let no_trash = req
        .headers()
        .get("X-No-Trashbin")
        .map(|val| matches!(val.to_str(), Ok("1")))
        .unwrap_or(false);

    let resource = resource_service.get_resource(&path).await?;

    let privileges = resource.get_user_privileges(&principal)?;
    if !privileges.has(&UserPrivilege::Write) {
        return Err(Error::Unauthorized.into());
    }

    if !resource.satisfies_if_match(&if_match) {
        // Precondition failed
        return Ok(HttpResponse::PreconditionFailed().finish());
    }
    if resource.satisfies_if_none_match(&if_none_match) {
        // Precondition failed
        return Ok(HttpResponse::PreconditionFailed().finish());
    }

    resource_service.delete_resource(&path, !no_trash).await?;

    Ok(HttpResponse::Ok().body(""))
}
