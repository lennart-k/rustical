use crate::depth_extractor::Depth;
use crate::namespace::Namespace;
use crate::resource::HandlePropfind;
use crate::resource::ResourceService;
use crate::xml::tag_list::TagList;
use crate::Error;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path};
use actix_web::{HttpRequest, HttpResponse};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use serde::Deserialize;
use serde::Serialize;

// This is not the final place for this struct
pub struct ServicePrefix(pub String);

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PropElement {
    #[serde(flatten)]
    pub prop: TagList,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PropfindType {
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
pub struct MultistatusElement<T1: Serialize, T2: Serialize> {
    #[serde(rename = "response")]
    pub responses: Vec<T1>,
    #[serde(rename = "response")]
    pub member_responses: Vec<T2>,
    #[serde(rename = "@xmlns")]
    pub ns_dav: &'static str,
    #[serde(rename = "@xmlns:C")]
    pub ns_caldav: &'static str,
    #[serde(rename = "@xmlns:IC")]
    pub ns_ical: &'static str,
}

pub async fn route_propfind<A: CheckAuthentication, R: ResourceService + ?Sized>(
    path: Path<R::PathComponents>,
    body: String,
    req: HttpRequest,
    prefix: Data<ServicePrefix>,
    auth: AuthInfoExtractor<A>,
    depth: Depth,
) -> Result<HttpResponse, R::Error> {
    let auth_info = auth.inner;
    let prefix = prefix.0.to_owned();
    let path_components = path.into_inner();

    let resource_service = R::new(req, auth_info.clone(), path_components.clone()).await?;

    // A request body is optional. If empty we MUST return all props
    let propfind: PropfindElement = if !body.is_empty() {
        quick_xml::de::from_str(&body).map_err(Error::XmlDecodeError)?
    } else {
        PropfindElement {
            prop: PropfindType::Allprop,
        }
    };

    let props = match propfind.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            // TODO: Implement
            return Err(Error::InternalError.into());
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

    let mut output = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n".to_owned();
    let mut ser = quick_xml::se::Serializer::new(&mut output);
    ser.indent(' ', 4);
    MultistatusElement {
        responses: vec![response],
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
