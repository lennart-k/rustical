use actix_web::http::header::ContentType;
use actix_web::http::Method;
use actix_web::web::{self, Data, Path};
use actix_web::{guard, HttpRequest, HttpResponse, Responder};
use depth_extractor::Depth;
use error::Error;
use namespace::Namespace;
use propfind::{generate_multistatus, parse_propfind};
use quick_xml::events::BytesText;
use resource::{HandlePropfind, Resource};
use resources::calendar::CalendarResource;
use resources::event::EventResource;
use resources::principal::PrincipalCalendarsResource;
use resources::root::RootResource;
use routes::{calendar, event};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_store::calendar::CalendarStore;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod depth_extractor;
pub mod error;
pub mod namespace;
mod propfind;
pub mod proptypes;
pub mod resource;
pub mod resources;
pub mod routes;

pub struct CalDavContext<C: CalendarStore> {
    pub prefix: String,
    pub store: Arc<RwLock<C>>,
}

pub fn configure_well_known(cfg: &mut web::ServiceConfig, caldav_root: String) {
    cfg.service(web::redirect("/caldav", caldav_root).permanent());
}

pub fn configure_dav<A: CheckAuthentication, C: CalendarStore>(
    cfg: &mut web::ServiceConfig,
    prefix: String,
    auth: Arc<A>,
    store: Arc<RwLock<C>>,
) {
    let propfind_method = || Method::from_str("PROPFIND").unwrap();
    let report_method = || Method::from_str("REPORT").unwrap();
    let mkcol_method = || Method::from_str("MKCOL").unwrap();

    cfg.app_data(Data::new(CalDavContext {
        prefix,
        store: store.clone(),
    }))
    .app_data(Data::from(store.clone()))
    .app_data(Data::from(auth))
    .service(
        web::resource("{path:.*}")
            // Without the guard this service would handle all requests
            .guard(guard::Method(Method::OPTIONS))
            .to(options_handler),
    )
    .service(
        web::resource("").route(web::method(propfind_method()).to(route_new_propfind::<
            A,
            RootResource,
            C,
        >)),
    )
    .service(
        web::resource("/{principal}").route(
            web::method(propfind_method()).to(route_new_propfind::<
                A,
                PrincipalCalendarsResource<C>,
                C,
            >),
        ),
    )
    .service(
        web::resource("/{principal}/{calendar}")
            .route(web::method(report_method()).to(calendar::route_report_calendar::<A, C>))
            .route(
                web::method(propfind_method()).to(route_new_propfind::<A, CalendarResource<C>, C>),
            )
            .route(web::method(mkcol_method()).to(calendar::route_mkcol_calendar::<A, C>)),
    )
    .service(
        web::resource("/{principal}/{calendar}/{event}")
            .route(web::method(propfind_method()).to(route_new_propfind::<A, EventResource, C>))
            .route(web::method(propfind_method()).to(route_new_propfind::<A, EventResource<C>, C>))
            .route(web::method(Method::DELETE).to(event::delete_event::<A, C>))
            .route(web::method(Method::GET).to(event::get_event::<A, C>))
            .route(web::method(Method::PUT).to(event::put_event::<A, C>)),
    );
}

async fn route_new_propfind<A: CheckAuthentication, R: Resource, C: CalendarStore>(
    path: Path<R::UriComponents>,
    body: String,
    req: HttpRequest,
    context: Data<CalDavContext<C>>,
    auth: AuthInfoExtractor<A>,
    depth: Depth,
) -> Result<HttpResponse, Error> {
    let props = parse_propfind(&body).map_err(|_e| Error::BadRequest)?;
    let req_path = req.path().to_string();
    let auth_info = auth.inner;

    let resource = R::acquire_from_request(
        req,
        auth_info,
        path.into_inner(),
        context.prefix.to_string(),
    )
    .await
    .map_err(|_e| Error::InternalError)?;

    let mut responses = vec![resource
        .propfind(props.clone())
        .map_err(|_e| Error::InternalError)?];

    if depth != Depth::Zero {
        for member in resource
            .get_members()
            .await
            .map_err(|_e| Error::InternalError)?
        {
            responses.push(
                member
                    .propfind(props.clone())
                    .map_err(|_e| Error::InternalError)?,
            );
        }
    }

    let output = generate_multistatus(
        vec![Namespace::Dav, Namespace::CalDAV, Namespace::ICal],
        |writer| {
            for response in responses {
                writer.write_event(quick_xml::events::Event::Text(BytesText::from_escaped(
                    response,
                )))?;
            }
            Ok(())
        },
    )
    .map_err(|_e| Error::InternalError)?;

    Ok(HttpResponse::MultiStatus()
        .content_type(ContentType::xml())
        .body(output))
}

async fn options_handler() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((
            "Allow",
            "OPTIONS, GET, HEAD, POST, PUT, REPORT, PROPFIND, PROPPATCH, MKCOL",
        ))
        .insert_header((
            "DAV",
            "1, 2, 3, calendar-access, extended-mkcol",
            // "1, 2, 3, calendar-access, addressbook, extended-mkcol",
        ))
        .body("options")
}
