use crate::resources::event::EventResource;
use crate::{CalDavContext, Error};
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path};
use actix_web::{HttpRequest, HttpResponse};
use anyhow::Result;
use quick_xml::events::BytesText;
use roxmltree::{Node, NodeType};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_dav::namespace::Namespace;
use rustical_dav::resource::HandlePropfind;
use rustical_dav::xml_snippets::generate_multistatus;
use rustical_store::calendar::{Calendar, CalendarStore, Event};
use std::sync::Arc;
use tokio::sync::RwLock;

async fn _parse_filter(filter_node: &Node<'_, '_>) {
    for comp_filter_node in filter_node.children() {
        if comp_filter_node.tag_name().name() != "comp-filter" {
            dbg!("wtf", comp_filter_node.tag_name().name());
            continue;
        }

        for filter in filter_node.children() {
            match filter.tag_name().name() {
                // <time-range start=\"20230804T125257Z\" end=\"20231013T125257Z\"/
                "time-range" => {}
                _ => {
                    dbg!("unknown filter", filter.tag_name());
                }
            }
        }
    }
}

async fn handle_report_calendar_query<C: CalendarStore>(
    query_node: Node<'_, '_>,
    request: HttpRequest,
    events: Vec<Event>,
    cal_store: Arc<RwLock<C>>,
) -> Result<HttpResponse, Error> {
    let prop_node = query_node
        .children()
        .find(|n| n.node_type() == NodeType::Element && n.tag_name().name() == "prop")
        .ok_or(Error::BadRequest)?;

    let props: Vec<&str> = prop_node
        .children()
        .map(|node| node.tag_name().name())
        .collect();
    let output = generate_multistatus(vec![Namespace::Dav, Namespace::CalDAV], |writer| {
        for event in events {
            let path = format!("{}/{}", request.path(), event.get_uid());
            let event_resource = EventResource {
                cal_store: cal_store.clone(),
                path: path.clone(),
                event,
            };
            // TODO: proper error handling
            let propfind_result = event_resource
                .propfind(props.clone())
                .map_err(|_e| quick_xml::Error::TextNotFound)?;

            writer.write_event(quick_xml::events::Event::Text(BytesText::from_escaped(
                propfind_result,
            )))?;
        }
        Ok(())
    })
    .map_err(|_e| Error::InternalError)?;

    Ok(HttpResponse::MultiStatus()
        .content_type(ContentType::xml())
        .body(output))
}

pub async fn route_report_calendar<A: CheckAuthentication, C: CalendarStore>(
    context: Data<CalDavContext<C>>,
    body: String,
    path: Path<(String, String)>,
    request: HttpRequest,
    _auth: AuthInfoExtractor<A>,
) -> Result<HttpResponse, Error> {
    // TODO: Check authorization
    let (_principal, cid) = path.into_inner();

    let doc = roxmltree::Document::parse(&body).map_err(|_e| Error::BadRequest)?;
    let query_node = doc.root_element();
    let events = context.store.read().await.get_events(&cid).await.unwrap();

    dbg!(&body);

    // TODO: implement filtering
    match query_node.tag_name().name() {
        "calendar-query" => {}
        "calendar-multiget" => {}
        _ => return Err(Error::BadRequest),
    };
    handle_report_calendar_query(query_node, request, events, context.store.clone()).await
}

pub async fn handle_mkcol_calendar_set<C: CalendarStore>(
    store: &RwLock<C>,
    prop_node: Node<'_, '_>,
    cid: String,
    owner: String,
) -> Result<()> {
    let mut cal = Calendar {
        owner,
        id: cid.clone(),
        ..Default::default()
    };
    for prop in prop_node.children() {
        match prop.tag_name().name() {
            "displayname" => {
                cal.name = prop.text().map(|s| s.to_string());
            }
            "timezone" => {
                cal.timezone = prop.text().map(|s| s.to_string());
            }
            "calendar-color" => {
                cal.color = prop.text().map(|s| s.to_string());
            }
            "calendar-description" => {
                cal.description = prop.text().map(|s| s.to_string());
            }
            "calendar-timezone" => {
                cal.timezone = prop.text().map(|s| s.to_string());
            }
            _ => {
                println!("unsupported mkcol tag: {}", prop.tag_name().name())
            }
        }
    }

    store.write().await.insert_calendar(cid, cal).await?;
    Ok(())
}

pub async fn route_mkcol_calendar<A: CheckAuthentication, C: CalendarStore>(
    path: Path<(String, String)>,
    body: String,
    auth: AuthInfoExtractor<A>,
    context: Data<CalDavContext<C>>,
) -> Result<HttpResponse, Error> {
    let (_principal, cid) = path.into_inner();
    let doc = roxmltree::Document::parse(&body).map_err(|_e| Error::BadRequest)?;
    let mkcol_node = doc.root_element();
    match mkcol_node.tag_name().name() {
        "mkcol" => {}
        _ => return Err(Error::BadRequest),
    }

    // TODO: Why does the spec (rfc5689) allow multiple <set/> elements but only one resource? :/
    // Well, for now just bother with the first one
    let set_node = mkcol_node.first_element_child().ok_or(Error::BadRequest)?;
    match set_node.tag_name().name() {
        "set" => {}
        _ => return Err(Error::BadRequest),
    }

    let prop_node = set_node.first_element_child().ok_or(Error::BadRequest)?;
    if prop_node.tag_name().name() != "prop" {
        return Err(Error::BadRequest);
    }
    handle_mkcol_calendar_set(
        &context.store,
        prop_node,
        cid.clone(),
        auth.inner.user_id.clone(),
    )
    .await
    .map_err(|_e| Error::InternalError)?;

    Ok(HttpResponse::Created().body(""))
}
