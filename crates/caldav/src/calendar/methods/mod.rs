use crate::event::resource::EventFile;
use crate::CalDavContext;
use crate::Error;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use anyhow::Result;
use roxmltree::{Node, NodeType};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_dav::namespace::Namespace;
use rustical_dav::propfind::ServicePrefix;
use rustical_dav::resource::HandlePropfind;
use rustical_dav::xml_snippets::generate_multistatus;
use rustical_store::calendar::CalendarStore;
use rustical_store::event::Event;

pub mod mkcalendar;

async fn handle_report_calendar_query(
    query_node: Node<'_, '_>,
    events: Vec<Event>,
    prefix: &str,
) -> Result<HttpResponse, Error> {
    let prop_node = query_node
        .children()
        .find(|n| n.node_type() == NodeType::Element && n.tag_name().name() == "prop")
        .ok_or(Error::BadRequest)?;

    let props: Vec<&str> = prop_node
        .children()
        .map(|node| node.tag_name().name())
        .collect();

    let event_files: Vec<_> = events
        .into_iter()
        .map(|event| {
            // TODO: fix
            // let path = format!("{}/{}", request.path(), event.get_uid());
            EventFile {
                event, // cal_store: cal_store.clone(),
            }
        })
        .collect();
    let mut event_responses = Vec::new();
    for event_file in event_files {
        event_responses.push(event_file.propfind(prefix, props.clone()).await?);
    }
    // let event_results: Result<Vec<_>, _> = event_files
    //     .iter()
    //     .map(|ev| ev.propfind(props.clone()))
    //     .collect();
    // let event_responses = event_results?;

    let output = generate_multistatus(vec![Namespace::Dav, Namespace::CalDAV], |writer| {
        for result in event_responses {
            writer
                .write_serializable("response", &result)
                .map_err(|_e| quick_xml::Error::TextNotFound)?;
        }
        Ok(())
    })?;

    Ok(HttpResponse::MultiStatus()
        .content_type(ContentType::xml())
        .body(output))
}

pub async fn route_report_calendar<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    context: Data<CalDavContext<C>>,
    body: String,
    path: Path<(String, String)>,
    _auth: AuthInfoExtractor<A>,
    prefix: Data<ServicePrefix>,
) -> Result<HttpResponse, Error> {
    // TODO: Check authorization
    let (_principal, cid) = path.into_inner();
    let prefix = &prefix.0;

    let doc = roxmltree::Document::parse(&body).map_err(|_e| Error::BadRequest)?;
    let query_node = doc.root_element();
    let events = context.store.read().await.get_events(&cid).await.unwrap();

    // TODO: implement filtering
    match query_node.tag_name().name() {
        "calendar-query" => {}
        "calendar-multiget" => {}
        _ => return Err(Error::BadRequest),
    };
    handle_report_calendar_query(query_node, events, prefix).await
}

pub async fn delete_calendar<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    context: Data<CalDavContext<C>>,
    path: Path<(String, String)>,
    auth: AuthInfoExtractor<A>,
) -> Result<HttpResponse, Error> {
    let _user = auth.inner.user_id;
    // TODO: verify whether user is authorized
    let (_principal, cid) = path.into_inner();
    context.store.write().await.delete_calendar(&cid).await?;

    Ok(HttpResponse::Ok().body(""))
}
