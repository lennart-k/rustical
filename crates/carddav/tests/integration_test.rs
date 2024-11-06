use actix_web::{web, App};
use rustical_carddav::configure_dav;

#[actix_web::test]
async fn test_asd() {
    let app = App::new().service(web::scope("/carddav").configure(
        |config| configure_dav(config, auth_provider, store)
    ));
    let app = actix_web::test::init_service()
    assert!(false);
}
