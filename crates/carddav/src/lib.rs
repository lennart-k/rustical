use actix_web::{
    dev::Service,
    http::{
        header::{HeaderName, HeaderValue},
        Method, StatusCode,
    },
    web::{self, Data},
};
use address_object::resource::AddressObjectResourceService;
use addressbook::resource::AddressbookResourceService;
pub use error::Error;
use futures_util::FutureExt;
use principal::{PrincipalResource, PrincipalResourceService};
use rustical_dav::resource::ResourceService;
use rustical_dav::resources::RootResourceService;
use rustical_store::{
    auth::{AuthenticationMiddleware, AuthenticationProvider},
    AddressbookStore,
};
use std::sync::Arc;

pub mod address_object;
pub mod addressbook;
pub mod error;
pub mod principal;

pub fn configure_well_known(cfg: &mut web::ServiceConfig, carddav_root: String) {
    cfg.service(web::redirect("/carddav", carddav_root).permanent());
}

pub fn configure_dav<AP: AuthenticationProvider, A: AddressbookStore + ?Sized>(
    cfg: &mut web::ServiceConfig,
    auth_provider: Arc<AP>,
    store: Arc<A>,
) {
    cfg.service(
        web::scope("")
            .wrap(AuthenticationMiddleware::new(auth_provider))
            .wrap_fn(|req, srv| {
                // Middleware to set the DAV header
                // Could be more elegant if actix_web::guard::RegisteredMethods was public :(
                let method = req.method().clone();
                srv.call(req).map(move |res| {
                    if method == Method::OPTIONS {
                        return res.map(|mut response| {
                            if response.status() == StatusCode::METHOD_NOT_ALLOWED {
                                response.headers_mut().insert(
                                    HeaderName::from_static("dav"),
                                    HeaderValue::from_static(
                                        "1, 2, 3, access-control, addressbook, extended-mkcol",
                                    ),
                                );
                                *response.response_mut().status_mut() = StatusCode::OK;
                            }
                            response
                        });
                    }
                    res
                })
            })
            .app_data(Data::from(store.clone()))
            .service(RootResourceService::<PrincipalResource>::actix_resource())
            .service(
                web::scope("/user").service(
                    web::scope("/{principal}")
                        .service(PrincipalResourceService::<A>::actix_resource())
                        .service(
                            web::scope("/{addressbook}")
                                .service(AddressbookResourceService::<A>::actix_resource())
                                .service(
                                    web::scope("/{object}").service(
                                        AddressObjectResourceService::<A>::actix_resource(),
                                    ),
                                ),
                        ),
                ),
            ),
    );
}
