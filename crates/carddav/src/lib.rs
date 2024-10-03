use actix_web::{web, HttpResponse, Responder};

pub fn configure_well_known(cfg: &mut web::ServiceConfig, carddav_root: String) {
    cfg.service(web::redirect("/carddav", carddav_root).permanent());
}

pub fn configure_dav(_cfg: &mut web::ServiceConfig, _prefix: String) {}

pub async fn options_handler() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((
            "Allow",
            "OPTIONS, GET, HEAD, POST, PUT, REPORT, PROPFIND, PROPPATCH, MKCOL",
        ))
        .insert_header(("DAV", "1, 2, 3, addressbook, extended-mkcol"))
        .body("options")
}
