use crate::namespace::Namespace;
use crate::propfind::MultistatusElement;
use crate::resource::InvalidProperty;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::resource::{PropstatElement, PropstatResponseElement, PropstatType};
use crate::xml::tag_list::TagList;
use crate::xml::tag_name::TagName;
use crate::Error;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{web::Path, HttpRequest, HttpResponse};
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use serde::{Deserialize, Serialize};

// https://docs.rs/quick-xml/latest/quick_xml/de/index.html#normal-enum-variant
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
struct PropertyElement<T> {
    #[serde(rename = "$value")]
    prop: T,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
struct SetPropertyElement<T> {
    prop: PropertyElement<T>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
struct RemovePropertyElement {
    #[serde(rename = "$value")]
    prop: TagName,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
struct PropertyupdateElement<T> {
    #[serde(default = "Vec::new")]
    set: Vec<SetPropertyElement<T>>,
    #[serde(default = "Vec::new")]
    remove: Vec<RemovePropertyElement>,
}

pub async fn route_proppatch<A: CheckAuthentication, R: ResourceService + ?Sized>(
    path: Path<R::PathComponents>,
    body: String,
    req: HttpRequest,
    auth: AuthInfoExtractor<A>,
) -> Result<HttpResponse, R::Error> {
    let auth_info = auth.inner;
    let path_components = path.into_inner();
    let href = req.path().to_owned();
    let resource_service = R::new(req, auth_info.clone(), path_components.clone()).await?;

    // TODO: Implement remove!
    let PropertyupdateElement::<<R::File as Resource>::Prop> {
        set: set_els,
        remove: remove_els,
    } = quick_xml::de::from_str(&body).map_err(Error::XmlDecodeError)?;

    // Extract all property names without verification
    // Weird workaround because quick_xml doesn't allow untagged enums
    let propnames: Vec<String> = quick_xml::de::from_str::<PropertyupdateElement<TagName>>(&body)
        .map_err(Error::XmlDecodeError)?
        .set
        .into_iter()
        .map(|set_el| set_el.prop.prop.into())
        .collect();

    // Invalid properties
    let props_not_found: Vec<String> = propnames
        .iter()
        .zip(&set_els)
        .filter_map(
            |(
                name,
                SetPropertyElement {
                    prop: PropertyElement { prop },
                },
            )| {
                if prop.invalid_property() {
                    Some(name.to_string())
                } else {
                    None
                }
            },
        )
        .collect();

    // Filter out invalid props
    let set_props: Vec<<R::File as Resource>::Prop> = set_els
        .into_iter()
        .filter_map(
            |SetPropertyElement {
                 prop: PropertyElement { prop },
             }| {
                if prop.invalid_property() {
                    None
                } else {
                    Some(prop)
                }
            },
        )
        .collect();

    let mut resource = resource_service.get_file().await?;

    let mut props_ok = Vec::new();
    let mut props_conflict = Vec::new();

    for (prop, propname) in set_props.into_iter().zip(propnames) {
        match resource.set_prop(prop) {
            Ok(()) => {
                props_ok.push(propname);
            }
            Err(Error::PropReadOnly) => {
                props_conflict.push(propname);
            }
            Err(err) => {
                return Err(err.into());
            }
        };
    }

    if props_not_found.is_empty() && props_conflict.is_empty() {
        // Only save if no errors occured
        resource_service.save_file(resource).await?;
    }

    let mut output = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n".to_owned();
    let mut ser = quick_xml::se::Serializer::new(&mut output);
    ser.indent(' ', 4);
    MultistatusElement {
        responses: vec![PropstatResponseElement {
            href,
            propstat: vec![
                PropstatType::Normal(PropstatElement {
                    prop: TagList::from(props_ok),
                    status: format!("HTTP/1.1 {}", StatusCode::OK),
                }),
                PropstatType::NotFound(PropstatElement {
                    prop: TagList::from(props_not_found),
                    status: format!("HTTP/1.1 {}", StatusCode::NOT_FOUND),
                }),
                PropstatType::Conflict(PropstatElement {
                    prop: TagList::from(props_conflict),
                    status: format!("HTTP/1.1 {}", StatusCode::CONFLICT),
                }),
            ],
        }],
        // Dummy just for typing
        member_responses: Vec::<String>::new(),
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
