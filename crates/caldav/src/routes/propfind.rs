use crate::CalDavContext;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpRequest, HttpResponse};
use anyhow::{anyhow, Result};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_dav::depth_extractor::Depth;
use rustical_dav::namespace::Namespace;
use rustical_dav::resource::{HandlePropfind, Resource};
use rustical_dav::xml_snippets::generate_multistatus;
use rustical_store::calendar::CalendarStore;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid propfind request: {0}")]
    InvalidPropfind(&'static str),
    #[error("input is not valid xml")]
    ParsingError(#[from] roxmltree::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl actix_web::error::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::InvalidPropfind(_) => StatusCode::BAD_REQUEST,
            Self::ParsingError(_) => StatusCode::BAD_REQUEST,
            Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

pub fn parse_propfind(body: &str) -> Result<Vec<&str>, Error> {
    if body.is_empty() {
        // if body is empty, allprops must be returned (RFC 4918)
        return Ok(vec!["allprops"]);
    }
    let doc = roxmltree::Document::parse(body)?;

    let propfind_node = doc.root_element();
    if propfind_node.tag_name().name() != "propfind" {
        return Err(Error::InvalidPropfind("root tag is not <propfind>"));
    }

    let prop_node = if let Some(el) = propfind_node.first_element_child() {
        el
    } else {
        return Ok(Vec::new());
    };

    match prop_node.tag_name().name() {
        "prop" => Ok(prop_node
            .children()
            .filter(|node| node.is_element())
            .map(|node| node.tag_name().name())
            .collect()),
        _ => Err(Error::InvalidPropfind(
            "invalid tag in <propfind>, expected <prop>",
        )),
    }
}

pub async fn route_propfind<A: CheckAuthentication, R: Resource, C: CalendarStore + ?Sized>(
    path: Path<R::UriComponents>,
    body: String,
    req: HttpRequest,
    context: Data<CalDavContext<C>>,
    auth: AuthInfoExtractor<A>,
    depth: Depth,
) -> Result<HttpResponse, rustical_dav::error::Error> {
    // TODO: fix errors
    let props = parse_propfind(&body).map_err(|_e| anyhow!("propfing parsing error"))?;
    let auth_info = auth.inner;

    let resource = R::acquire_from_request(
        req,
        auth_info,
        path.into_inner(),
        context.prefix.to_string(),
    )
    .await?;

    let response = resource.propfind(props.clone())?;
    let members = resource.get_members().await?;
    let mut member_responses = Vec::new();

    if depth != Depth::Zero {
        for member in &members {
            member_responses.push(member.propfind(props.clone())?);
        }
    }

    let output = generate_multistatus(
        vec![Namespace::Dav, Namespace::CalDAV, Namespace::ICal],
        |writer| {
            writer
                .write_serializable("response", &response)
                .map_err(|_e| quick_xml::Error::TextNotFound)?;
            for response in member_responses {
                writer
                    .write_serializable("response", &response)
                    .map_err(|_e| quick_xml::Error::TextNotFound)?;
            }
            Ok(())
        },
    )?;

    Ok(HttpResponse::MultiStatus()
        .content_type(ContentType::xml())
        .body(output))
}

#[derive(Serialize)]
struct MultistatusElement<T1: Serialize, T2: Serialize> {
    #[serde(rename = "$value")]
    responses: Vec<T1>,
    #[serde(rename = "$value")]
    member_responses: Vec<T2>,
    #[serde(rename = "@xmlns")]
    ns_dav: &'static str,
    #[serde(rename = "@xmlns:C")]
    ns_caldav: &'static str,
    #[serde(rename = "@xmlns:IC")]
    ns_ical: &'static str,
}
