use crate::resource::ResourceService;
use actix_web::web::Path;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use rustical_store::auth::User;

pub async fn route_delete<R: ResourceService>(
    path_components: Path<R::PathComponents>,
    req: HttpRequest,
    _user: User,
) -> Result<impl Responder, R::Error> {
    let path_components = path_components.into_inner();

    let no_trash = req
        .headers()
        .get("X-No-Trashbin")
        .map(|val| matches!(val.to_str(), Ok("1")))
        .unwrap_or(false);

    let resource_service = R::new(&req, path_components.clone()).await?;
    resource_service.delete_resource(!no_trash).await?;

    Ok(HttpResponse::Ok().body(""))
}
