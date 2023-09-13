use crate::depth_extractor::Depth;
use crate::error::Error;
use crate::namespace::Namespace;
use crate::resource::{HandlePropfind, Resource};
use crate::xml_snippets::generate_multistatus;
use crate::CalDavContext;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path};
use actix_web::{HttpRequest, HttpResponse};
use anyhow::{anyhow, Result};
use quick_xml::events::BytesText;
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_store::calendar::CalendarStore;

fn parse_propfind(body: &str) -> Result<Vec<&str>> {
    if body.is_empty() {
        // if body is empty, allprops must be returned (RFC 4918)
        return Ok(vec!["allprops"]);
    }
    let doc = roxmltree::Document::parse(body)?;

    let propfind_node = doc.root_element();
    if propfind_node.tag_name().name() != "propfind" {
        return Err(anyhow!("invalid tag"));
    }

    let prop_node = if let Some(el) = propfind_node.first_element_child() {
        el
    } else {
        return Ok(Vec::new());
    };

    let props = match prop_node.tag_name().name() {
        "prop" => Ok(prop_node
            .children()
            .map(|node| node.tag_name().name())
            .collect()),
        _ => Err(anyhow!("invalid prop tag")),
    };
    dbg!(body, &props);
    props
}

pub async fn route_propfind<A: CheckAuthentication, R: Resource, C: CalendarStore>(
    path: Path<R::UriComponents>,
    body: String,
    req: HttpRequest,
    context: Data<CalDavContext<C>>,
    auth: AuthInfoExtractor<A>,
    depth: Depth,
) -> Result<HttpResponse, Error> {
    let props = parse_propfind(&body).map_err(|_e| Error::BadRequest)?;
    // let req_path = req.path().to_string();
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
