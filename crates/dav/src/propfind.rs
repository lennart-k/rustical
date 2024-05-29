use crate::depth_extractor::Depth;
use crate::namespace::Namespace;
use crate::resource::HandlePropfind;
use crate::resource::ResourceService;
use crate::xml::tag_list::TagList;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path};
use actix_web::{HttpRequest, HttpResponse};
use anyhow::Result;
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use serde::Deserialize;
use serde::Serialize;

// This is not the final place for this struct
pub struct ServicePrefix(pub String);

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
struct PropElement {
    #[serde(flatten)]
    prop: TagList,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
enum PropfindType {
    Propname,
    Allprop,
    Prop(PropElement),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
struct PropfindElement {
    #[serde(rename = "$value")]
    prop: PropfindType,
}

#[derive(Serialize)]
#[serde(rename = "multistatus")]
struct MultistatusElement<T1: Serialize, T2: Serialize> {
    response: T1,
    #[serde(rename = "response")]
    member_responses: Vec<T2>,
    #[serde(rename = "@xmlns")]
    ns_dav: &'static str,
    #[serde(rename = "@xmlns:C")]
    ns_caldav: &'static str,
    #[serde(rename = "@xmlns:IC")]
    ns_ical: &'static str,
}

pub async fn route_propfind<A: CheckAuthentication, R: ResourceService + ?Sized>(
    path: Path<R::PathComponents>,
    body: String,
    req: HttpRequest,
    prefix: Data<ServicePrefix>,
    auth: AuthInfoExtractor<A>,
    depth: Depth,
) -> Result<HttpResponse, crate::error::Error> {
    let auth_info = auth.inner;
    let prefix = prefix.0.to_owned();
    let path_components = path.into_inner();

    let resource_service = R::new(req, auth_info.clone(), path_components.clone()).await?;

    let propfind: PropfindElement = quick_xml::de::from_str(&body).unwrap();
    let props = match propfind.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            // TODO: Implement
            return Err(crate::error::Error::InternalError);
        }
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags.into(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let mut member_responses = Vec::new();
    if depth != Depth::Zero {
        for member in resource_service.get_members(auth_info).await? {
            member_responses.push(member.propfind(&prefix, props.clone()).await?);
        }
    }

    let resource = resource_service.get_file().await?;
    let response = resource.propfind(&prefix, props).await?;

    let mut output = String::new();
    let mut ser = quick_xml::se::Serializer::new(&mut output);
    ser.indent(' ', 4);
    MultistatusElement {
        response,
        member_responses,
        ns_dav: Namespace::Dav.as_str(),
        ns_caldav: Namespace::CalDAV.as_str(),
        ns_ical: Namespace::ICal.as_str(),
    }
    .serialize(ser)
    .unwrap();

    Ok(HttpResponse::MultiStatus()
        .content_type(ContentType::xml())
        .body(output))
}
