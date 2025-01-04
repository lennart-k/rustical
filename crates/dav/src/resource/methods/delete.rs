use crate::privileges::UserPrivilege;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::Error;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use rustical_store::auth::User;

pub async fn route_delete<R: ResourceService>(
    path: Path<R::PathComponents>,
    req: HttpRequest,
    user: User,
    resource_service: Data<R>,
) -> Result<impl Responder, R::Error> {
    let no_trash = req
        .headers()
        .get("X-No-Trashbin")
        .map(|val| matches!(val.to_str(), Ok("1")))
        .unwrap_or(false);

    let resource = resource_service.get_resource(&path).await?;
    let privileges = resource.get_user_privileges(&user)?;
    if !privileges.has(&UserPrivilege::Write) {
        // TODO: Actually the spec wants us to look whether we have unbind access in the parent
        // collection
        return Err(Error::Unauthorized.into());
    }
    resource_service.delete_resource(&path, !no_trash).await?;

    Ok(HttpResponse::Ok().body(""))
}
