use crate::depth_extractor::Depth;
use crate::namespace::Namespace;
use crate::resource::HandlePropfind;
use crate::resource::ResourceService;
use crate::xml_snippets::generate_multistatus;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpRequest, HttpResponse};
use anyhow::{anyhow, Result};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use serde::Serialize;
use thiserror::Error;

// This is not the final place for this struct
pub struct ServicePrefix(pub String);

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

pub async fn handle_propfind<
    A: CheckAuthentication,
    R: ResourceService + ?Sized,
    // C: CalendarStore + ?Sized,
>(
    path: Path<R::PathComponents>,
    body: String,
    req: HttpRequest,
    prefix: Data<ServicePrefix>,
    auth: AuthInfoExtractor<A>,
    depth: Depth,
) -> Result<HttpResponse, crate::error::Error> {
    // TODO: fix errors
    let props = parse_propfind(&body).map_err(|_e| anyhow!("propfind parsing error"))?;
    let auth_info = auth.inner;
    let prefix = prefix.0.to_owned();
    let path_components = path.into_inner();

    let resource_service = R::new(req, auth_info.clone(), path_components.clone()).await?;

    let resource = resource_service.get_file().await?;
    let response = resource.propfind(&prefix, props.clone()).await?;
    let mut member_responses = Vec::new();

    if depth != Depth::Zero {
        for member in resource_service.get_members(auth_info).await? {
            member_responses.push(member.propfind(&prefix, props.clone()).await?);
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
