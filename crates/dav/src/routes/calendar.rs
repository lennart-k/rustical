use crate::namespace::Namespace;
use crate::propfind::{
    generate_multistatus, parse_propfind, write_invalid_props_response, write_propstat_response,
    write_resourcetype,
};
use crate::{CalDavContext, Error};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpRequest, HttpResponse};
use actix_web_httpauth::extractors::basic::BasicAuth;
use anyhow::Result;
use quick_xml::events::BytesText;
use quick_xml::Writer;
use roxmltree::{Node, NodeType};
use rustical_store::calendar::{Calendar, CalendarStore, Event};
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

async fn handle_report_calendar_query(
    query_node: Node<'_, '_>,
    request: HttpRequest,
    events: Vec<Event>,
) -> Result<HttpResponse, Error> {
    let prop_node = query_node
        .children()
        .find(|n| n.node_type() == NodeType::Element && n.tag_name().name() == "prop")
        .ok_or(Error::BadRequest)?;

    let props: Arc<HashSet<&str>> = Arc::new(
        prop_node
            .children()
            .map(|node| node.tag_name().name())
            .collect(),
    );
    let output = generate_multistatus(vec![Namespace::Dav, Namespace::CalDAV], |writer| {
        for event in events {
            write_propstat_response(
                writer,
                &format!("{}/{}", request.path(), event.get_uid()),
                StatusCode::OK,
                |writer| {
                    for prop in props.deref() {
                        match *prop {
                            "getetag" => {
                                writer
                                    .create_element("getetag")
                                    .write_text_content(BytesText::new(&event.get_etag()))?;
                            }
                            "calendar-data" => {
                                writer
                                    .create_element("C:calendar-data")
                                    .write_text_content(BytesText::new(event.to_ics()))?;
                            }
                            _ => {}
                        }
                    }
                    Ok(())
                },
            )?;
        }
        Ok(())
    })
    .map_err(|_e| Error::InternalError)?;

    Ok(HttpResponse::MultiStatus()
        .content_type(ContentType::xml())
        .body(output))
}

pub async fn route_report_calendar<C: CalendarStore>(
    context: Data<CalDavContext<C>>,
    body: String,
    path: Path<(String, String)>,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (_principal, cid) = path.into_inner();

    let doc = roxmltree::Document::parse(&body).map_err(|_e| Error::InternalError)?;
    let query_node = doc.root_element();
    let events = context.store.read().await.get_events(&cid).await.unwrap();

    // TODO: implement filtering
    match query_node.tag_name().name() {
        "calendar-query" => {}
        "calendar-multiget" => {}
        _ => return Err(Error::BadRequest),
    };
    handle_report_calendar_query(query_node, request, events).await
}

pub fn generate_propfind_calendar_response(
    props: Vec<&str>,
    principal: &str,
    path: &str,
    prefix: &str,
    calendar: &Calendar,
) -> Result<String> {
    let mut props = props;
    if props.contains(&"allprops") {
        props.extend(
            [
                "resourcetype",
                "current-user-principal",
                "displayname",
                "supported-calendar-component-set",
                "getcontenttype",
                "calendar-description",
            ]
            .iter(),
        );
    }

    let mut invalid_props = Vec::<&str>::new();
    let mut output_buffer = Vec::new();
    let mut writer = Writer::new_with_indent(&mut output_buffer, b' ', 2);

    write_propstat_response(&mut writer, path, StatusCode::OK, |writer| {
        for prop in props {
            match prop {
                "resourcetype" => write_resourcetype(writer, vec!["C:calendar", "collection"])?,
                "current-user-principal" => {
                    writer
                        .create_element("current-user-principal")
                        .write_inner_content(|writer| {
                            writer
                                .create_element("href")
                                .write_text_content(BytesText::new(&format!(
                                    "{prefix}/{principal}/",
                                )))?;
                            Ok(())
                        })?;
                }
                "displayname" => {
                    let el = writer.create_element("displayname");
                    if let Some(name) = calendar.clone().name {
                        el.write_text_content(BytesText::new(&name))?;
                    } else {
                        el.write_empty()?;
                    }
                }
                "supported-calendar-component-set" => {
                    writer
                        .create_element("C:supported-calendar-component-set")
                        .write_inner_content(|writer| {
                            writer
                                .create_element("C:comp")
                                .with_attribute(("name", "VEVENT"))
                                .write_empty()?;
                            Ok(())
                        })?;
                }
                "getcontenttype" => {
                    writer
                        .create_element("getcontenttype")
                        .write_text_content(BytesText::new("text/calendar"))?;
                }
                "allprops" => {}
                _ => invalid_props.push(prop),
            };
        }
        Ok(())
    })?;

    write_invalid_props_response(&mut writer, path, invalid_props)?;
    Ok(std::str::from_utf8(&output_buffer)?.to_string())
}

pub async fn route_propfind_calendar<C: CalendarStore>(
    path: Path<(String, String)>,
    body: String,
    request: HttpRequest,
    auth: BasicAuth,
    context: Data<CalDavContext<C>>,
) -> Result<HttpResponse, Error> {
    let (_principal, cid) = path.into_inner();
    let calendar = context
        .store
        .read()
        .await
        .get_calendar(&cid)
        .await
        .map_err(|_e| Error::InternalError)?;

    let props = parse_propfind(&body).map_err(|_e| Error::BadRequest)?;

    let responses_string = generate_propfind_calendar_response(
        props.clone(),
        auth.user_id(),
        request.path(),
        &context.prefix,
        &calendar,
    )
    .map_err(|_e| Error::InternalError)?;

    let output = generate_multistatus(vec![Namespace::Dav, Namespace::CalDAV], |writer| {
        writer.write_event(quick_xml::events::Event::Text(BytesText::from_escaped(
            responses_string,
        )))?;
        Ok(())
    })
    .map_err(|_e| Error::InternalError)?;

    Ok(HttpResponse::MultiStatus()
        .content_type(ContentType::xml())
        .body(output))
}
