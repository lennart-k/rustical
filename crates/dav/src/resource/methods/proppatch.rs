use crate::privileges::UserPrivilege;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::xml::multistatus::{PropstatElement, PropstatWrapper, ResponseElement};
use crate::xml::MultistatusElement;
use crate::xml::TagList;
use crate::Error;
use actix_web::http::StatusCode;
use actix_web::{web::Path, HttpRequest};
use rustical_store::auth::User;
use rustical_xml::Unparsed;
use rustical_xml::XmlDeserialize;
use rustical_xml::XmlDocument;
use rustical_xml::XmlRootTag;
use std::str::FromStr;
use tracing::instrument;
use tracing_actix_web::RootSpan;

#[derive(XmlDeserialize, Clone, Debug)]
#[xml(untagged)]
enum SetPropertyPropWrapper<T: XmlDeserialize> {
    Valid(T),
    Invalid(Unparsed),
}

// We are <prop>
#[derive(XmlDeserialize, Clone, Debug)]
struct SetPropertyPropWrapperWrapper<T: XmlDeserialize> {
    #[xml(ty = "untagged")]
    property: SetPropertyPropWrapper<T>,
}

// We are <set>
#[derive(XmlDeserialize, Clone, Debug)]
struct SetPropertyElement<T: XmlDeserialize> {
    prop: T,
}

#[derive(XmlDeserialize, Clone, Debug)]
struct TagName {
    #[xml(ty = "tag_name")]
    name: String,
}

#[derive(XmlDeserialize, Clone, Debug)]
struct PropertyElement {
    #[xml(ty = "untagged")]
    property: TagName,
}

#[derive(XmlDeserialize, Clone, Debug)]
struct RemovePropertyElement {
    prop: PropertyElement,
}

#[derive(XmlDeserialize, Clone, Debug)]
enum Operation<T: XmlDeserialize> {
    Set(SetPropertyElement<T>),
    Remove(RemovePropertyElement),
}

#[derive(XmlDeserialize, XmlRootTag, Clone, Debug)]
#[xml(root = b"propertyupdate")]
struct PropertyupdateElement<T: XmlDeserialize> {
    #[xml(ty = "untagged", flatten)]
    operations: Vec<Operation<T>>,
}

#[instrument(parent = root_span.id(), skip(path, req, root_span))]
pub(crate) async fn route_proppatch<R: ResourceService>(
    path: Path<R::PathComponents>,
    body: String,
    req: HttpRequest,
    user: User,
    root_span: RootSpan,
) -> Result<MultistatusElement<String, String>, R::Error> {
    let path_components = path.into_inner();
    let href = req.path().to_owned();
    let resource_service = R::new(&req, path_components.clone()).await?;

    // Extract operations
    let PropertyupdateElement::<SetPropertyPropWrapperWrapper<<R::Resource as Resource>::Prop>> {
        operations,
    } = XmlDocument::parse_str(&body).map_err(Error::XmlDeserializationError)?;

    let mut resource = resource_service.get_resource().await?;
    let privileges = resource.get_user_privileges(&user)?;
    if !privileges.has(&UserPrivilege::Write) {
        return Err(Error::Unauthorized.into());
    }

    let mut props_ok = Vec::new();
    let mut props_conflict = Vec::new();
    let mut props_not_found = Vec::new();

    for operation in operations.into_iter() {
        match operation {
            Operation::Set(SetPropertyElement { prop }) => {
                match prop.property {
                    SetPropertyPropWrapper::Valid(prop) => {
                        let propname: <R::Resource as Resource>::PropName = prop.clone().into();
                        let propname: &str = propname.into();
                        match resource.set_prop(prop) {
                            Ok(()) => props_ok.push(propname.to_owned()),
                            Err(Error::PropReadOnly) => props_conflict.push(propname.to_owned()),
                            Err(err) => return Err(err.into()),
                        };
                    }
                    SetPropertyPropWrapper::Invalid(invalid) => {
                        let propname = invalid.tag_name();
                        if <R::Resource as Resource>::list_props().contains(&propname.as_str()) {
                            // This happens in following cases:
                            // - read-only properties with #[serde(skip_deserializing)]
                            // - internal properties
                            props_conflict.push(propname)
                        } else {
                            props_not_found.push(propname);
                        }
                    }
                }
            }
            Operation::Remove(remove_el) => {
                let propname = remove_el.prop.property.name;
                match <<R::Resource as Resource>::PropName as FromStr>::from_str(&propname) {
                    Ok(prop) => match resource.remove_prop(&prop) {
                        Ok(()) => props_ok.push(propname),
                        Err(Error::PropReadOnly) => props_conflict.push(propname),
                        Err(err) => return Err(err.into()),
                    },
                    // I guess removing a nonexisting property should be successful :)
                    Err(_) => props_ok.push(propname),
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
                    status: StatusCode::OK,
                }),
                PropstatWrapper::TagList(PropstatElement {
                    prop: TagList::from(props_not_found),
                    status: StatusCode::NOT_FOUND,
                }),
                PropstatWrapper::TagList(PropstatElement {
                    prop: TagList::from(props_conflict),
                    status: StatusCode::CONFLICT,
                }),
            ],
            ..Default::default()
        }],
        ..Default::default()
    })
}
