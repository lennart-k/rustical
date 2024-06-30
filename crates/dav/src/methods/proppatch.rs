use std::str::FromStr;

use crate::namespace::Namespace;
use crate::resource::InvalidProperty;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::resource::{PropstatElement, PropstatResponseElement, PropstatType};
use crate::xml::MultistatusElement;
use crate::xml::TagList;
use crate::xml::TagName;
use crate::Error;
use actix_web::http::StatusCode;
use actix_web::Responder;
use actix_web::{web::Path, HttpRequest};
use log::debug;
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
    prop: PropertyElement<TagName>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
enum Operation<T> {
    Set(SetPropertyElement<T>),
    Remove(RemovePropertyElement),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
struct PropertyupdateElement<T> {
    #[serde(rename = "$value", default = "Vec::new")]
    operations: Vec<Operation<T>>,
}

pub async fn route_proppatch<A: CheckAuthentication, R: ResourceService + ?Sized>(
    path: Path<R::PathComponents>,
    body: String,
    req: HttpRequest,
    auth: AuthInfoExtractor<A>,
) -> Result<impl Responder, R::Error> {
    let auth_info = auth.inner;
    let path_components = path.into_inner();
    let href = req.path().to_owned();
    let resource_service = R::new(&req, &auth_info, path_components.clone()).await?;

    debug!("{body}");

    let PropertyupdateElement::<<R::File as Resource>::Prop> { operations } =
        quick_xml::de::from_str(&body).map_err(Error::XmlDecodeError)?;

    // Extract all set property names without verification
    // Weird workaround because quick_xml doesn't allow untagged enums
    let propnames: Vec<String> = quick_xml::de::from_str::<PropertyupdateElement<TagName>>(&body)
        .map_err(Error::XmlDecodeError)?
        .operations
        .into_iter()
        .map(|op_el| match op_el {
            Operation::Set(set_el) => set_el.prop.prop.into(),
            // If we can't remove a nonexisting property then that's no big deal
            Operation::Remove(remove_el) => remove_el.prop.prop.into(),
        })
        .collect();

    let mut resource = resource_service.get_file().await?;

    let mut props_ok = Vec::new();
    let mut props_conflict = Vec::new();
    let mut props_not_found = Vec::new();

    for (operation, propname) in operations.into_iter().zip(propnames) {
        match operation {
            Operation::Set(SetPropertyElement {
                prop: PropertyElement { prop },
            }) => {
                if prop.invalid_property() {
                    props_not_found.push(propname);
                    continue;
                }
                match resource.set_prop(prop) {
                    Ok(()) => {
                        props_ok.push(propname);
                    }
                    Err(Error::PropReadOnly) => {
                        props_conflict.push(propname);
                    }
                    Err(err) => {
                        // TODO: Think about error handling?
                        return Err(err.into());
                    }
                }
            }
            Operation::Remove(_remove_el) => {
                match <<R::File as Resource>::PropName as FromStr>::from_str(&propname) {
                    Ok(prop) => {
                        match resource.remove_prop(prop) {
                            Ok(()) => {
                                props_ok.push(propname);
                            }
                            Err(Error::PropReadOnly) => {
                                props_conflict.push(propname);
                            }
                            Err(err) => {
                                // TODO: Think about error handling?
                                return Err(err.into());
                            }
                        }
                    }
                    Err(_) => {
                        // I guess removing a nonexisting property should be successful :)
                        props_ok.push(propname);
                    }
                };
            }
        }
    }

    if props_not_found.is_empty() && props_conflict.is_empty() {
        // Only save if no errors occured
        resource_service.save_file(resource).await?;
    }

    Ok(MultistatusElement {
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
    })
}
