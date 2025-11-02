use crate::Error;
use crate::privileges::UserPrivilege;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::xml::MultistatusElement;
use crate::xml::TagList;
use crate::xml::multistatus::{PropstatElement, PropstatWrapper, ResponseElement};
use axum::extract::{OriginalUri, Path, State};
use http::StatusCode;
use quick_xml::name::Namespace;
use rustical_xml::NamespaceOwned;
use rustical_xml::PropName;
use rustical_xml::Unparsed;
use rustical_xml::XmlDeserialize;
use rustical_xml::XmlDocument;
use rustical_xml::XmlRootTag;
use std::str::FromStr;

#[derive(XmlDeserialize, Clone, Debug)]
#[xml(untagged)]
enum SetPropertyPropWrapper<T: XmlDeserialize> {
    Valid(T),
    Invalid(Unparsed),
}

// We are <prop>
#[derive(XmlDeserialize, Clone, Debug)]
struct SetPropertyPropWrapperWrapper<T: XmlDeserialize>(
    #[xml(ty = "untagged", flatten)] Vec<SetPropertyPropWrapper<T>>,
);

// We are <set>
#[derive(XmlDeserialize, Clone, Debug)]
struct SetPropertyElement<T: XmlDeserialize> {
    #[xml(ns = "crate::namespace::NS_DAV")]
    prop: SetPropertyPropWrapperWrapper<T>,
}

#[derive(XmlDeserialize, Clone, Debug)]
struct TagName(#[xml(ty = "tag_name")] String);

#[derive(XmlDeserialize, Clone, Debug)]
struct PropertyElement(#[xml(ty = "untagged", flatten)] Vec<TagName>);

#[derive(XmlDeserialize, Clone, Debug)]
struct RemovePropertyElement {
    #[xml(ns = "crate::namespace::NS_DAV")]
    prop: PropertyElement,
}

#[derive(XmlDeserialize, Clone, Debug)]
enum Operation<T: XmlDeserialize> {
    #[xml(ns = "crate::namespace::NS_DAV")]
    Set(SetPropertyElement<T>),
    #[xml(ns = "crate::namespace::NS_DAV")]
    Remove(RemovePropertyElement),
}

#[derive(XmlDeserialize, XmlRootTag, Clone, Debug)]
#[xml(root = "propertyupdate")]
#[xml(ns = "crate::namespace::NS_DAV")]
struct PropertyupdateElement<T: XmlDeserialize>(#[xml(ty = "untagged", flatten)] Vec<Operation<T>>);

pub async fn axum_route_proppatch<R: ResourceService>(
    Path(path): Path<R::PathComponents>,
    State(resource_service): State<R>,
    principal: R::Principal,
    uri: OriginalUri,
    body: String,
) -> Result<MultistatusElement<String, String>, R::Error> {
    route_proppatch(&path, uri.path(), &body, &principal, &resource_service).await
}

pub async fn route_proppatch<R: ResourceService>(
    path_components: &R::PathComponents,
    path: &str,
    body: &str,
    principal: &R::Principal,
    resource_service: &R,
) -> Result<MultistatusElement<String, String>, R::Error> {
    let href = path.to_owned();

    // Extract operations
    let PropertyupdateElement::<<R::Resource as Resource>::Prop>(operations) =
        XmlDocument::parse_str(body).map_err(Error::XmlError)?;

    let mut resource = resource_service
        .get_resource(path_components, false)
        .await?;
    let privileges = resource.get_user_privileges(principal)?;
    if !privileges.has(&UserPrivilege::Write) {
        return Err(Error::Unauthorized.into());
    }

    let mut props_ok = Vec::new();
    let mut props_conflict = Vec::new();
    let mut props_not_found = Vec::new();

    for operation in operations {
        match operation {
            Operation::Set(SetPropertyElement {
                prop: SetPropertyPropWrapperWrapper(properties),
            }) => {
                for property in properties {
                    match property {
                        SetPropertyPropWrapper::Valid(prop) => {
                            let propname: <<R::Resource as Resource>::Prop as PropName>::Names =
                                prop.clone().into();
                            let (ns, propname): (Option<Namespace>, &str) = propname.into();
                            match resource.set_prop(prop) {
                                Ok(()) => props_ok
                                    .push((ns.map(NamespaceOwned::from), propname.to_owned())),
                                Err(Error::PropReadOnly) => props_conflict
                                    .push((ns.map(NamespaceOwned::from), propname.to_owned())),
                                Err(err) => return Err(err.into()),
                            }
                        }
                        SetPropertyPropWrapper::Invalid(invalid) => {
                            let propname = invalid.tag_name();

                            if let Some(full_propname) = <R::Resource as Resource>::list_props()
                                .into_iter()
                                .find_map(|(ns, tag)| {
                                    if tag == propname.as_str() {
                                        Some((ns.map(NamespaceOwned::from), tag.to_owned()))
                                    } else {
                                        None
                                    }
                                })
                            {
                                // This happens in following cases:
                                // - read-only properties with #[serde(skip_deserializing)]
                                // - internal properties
                                props_conflict.push(full_propname);
                            } else {
                                props_not_found.push((None, propname));
                            }
                        }
                    }
                }
            }
            Operation::Remove(remove_el) => {
                for tagname in remove_el.prop.0 {
                    let propname = tagname.0;
                    match <<R::Resource as Resource>::Prop as PropName>::Names::from_str(&propname)
                    {
                        Ok(prop) => match resource.remove_prop(&prop) {
                            Ok(()) => props_ok.push((None, propname)),
                            Err(Error::PropReadOnly) => props_conflict.push({
                                let (ns, tag) = prop.into();
                                (ns.map(NamespaceOwned::from), tag.to_owned())
                            }),
                            Err(err) => return Err(err.into()),
                        },
                        // I guess removing a nonexisting property should be successful :)
                        Err(_) => props_ok.push((None, propname)),
                    }
                }
            }
        }
    }

    if props_not_found.is_empty() && props_conflict.is_empty() {
        // Only save if no errors occured
        resource_service
            .save_resource(path_components, resource)
            .await?;
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
