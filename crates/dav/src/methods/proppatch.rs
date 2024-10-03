use crate::resource::InvalidProperty;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::xml::multistatus::{PropstatElement, PropstatWrapper, ResponseElement};
use crate::xml::MultistatusElement;
use crate::xml::TagList;
use crate::xml::TagName;
use crate::Error;
use actix_web::http::StatusCode;
use actix_web::{web::Path, HttpRequest};
use log::debug;
use rustical_store::auth::User;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

pub async fn route_proppatch<R: ResourceService>(
    path: Path<R::PathComponents>,
    body: String,
    req: HttpRequest,
    user: User,
) -> Result<MultistatusElement<PropstatWrapper<String>, PropstatWrapper<String>>, R::Error> {
    let path_components = path.into_inner();
    let href = req.path().to_owned();
    let resource_service = R::new(&req, path_components.clone()).await?;

    debug!("{body}");

    let PropertyupdateElement::<<R::Resource as Resource>::Prop> { operations } =
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

    let mut resource = resource_service.get_resource(user.id).await?;

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
                match <<R::Resource as Resource>::PropName as FromStr>::from_str(&propname) {
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
        resource_service.save_resource(resource).await?;
    }

    Ok(MultistatusElement {
        responses: vec![ResponseElement {
            href,
            propstat: vec![
                PropstatWrapper::TagList(PropstatElement {
                    prop: TagList::from(props_ok),
                    status: format!("HTTP/1.1 {}", StatusCode::OK),
                }),
                PropstatWrapper::TagList(PropstatElement {
                    prop: TagList::from(props_not_found),
                    status: format!("HTTP/1.1 {}", StatusCode::NOT_FOUND),
                }),
                PropstatWrapper::TagList(PropstatElement {
                    prop: TagList::from(props_conflict),
                    status: format!("HTTP/1.1 {}", StatusCode::CONFLICT),
                }),
            ],
            ..Default::default()
        }],
        ..Default::default()
    })
}
