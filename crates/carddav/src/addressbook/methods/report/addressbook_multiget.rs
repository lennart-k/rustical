use crate::{
    address_object::resource::{AddressObjectProp, AddressObjectResource},
    principal::PrincipalResource,
    Error,
};
use actix_web::{
    dev::{Path, ResourceDef},
    http::StatusCode,
    HttpRequest,
};
use rustical_dav::{
    methods::propfind::{PropElement, PropfindType},
    resource::{CommonPropertiesProp, EitherProp, Resource},
    xml::{
        multistatus::{PropstatWrapper, ResponseElement},
        MultistatusElement,
    },
};
use rustical_store::{auth::User, AddressObject, AddressbookStore};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub struct AddressbookMultigetRequest {
    #[serde(flatten)]
    prop: PropfindType,
    href: Vec<String>,
}

pub async fn get_objects_addressbook_multiget<AS: AddressbookStore + ?Sized>(
    addressbook_multiget: &AddressbookMultigetRequest,
    principal_url: &str,
    principal: &str,
    addressbook_id: &str,
    store: &AS,
) -> Result<(Vec<AddressObject>, Vec<String>), Error> {
    let resource_def =
        ResourceDef::prefix(principal_url).join(&ResourceDef::new("/{addressbook_id}/{object_id}"));

    let mut result = vec![];
    let mut not_found = vec![];

    for href in &addressbook_multiget.href {
        let mut path = Path::new(href.as_str());
        if !resource_def.capture_match_info(&mut path) {
            not_found.push(href.to_owned());
        };
        if path.get("addressbook_id").unwrap() != addressbook_id {
            not_found.push(href.to_owned());
        }
        let object_id = path.get("object_id").unwrap();
        match store.get_object(principal, addressbook_id, object_id).await {
            Ok(object) => result.push(object),
            Err(rustical_store::Error::NotFound) => not_found.push(href.to_owned()),
            // TODO: Maybe add error handling on a per-object basis
            Err(err) => return Err(err.into()),
        };
    }

    Ok((result, not_found))
}

pub async fn handle_addressbook_multiget<AS: AddressbookStore + ?Sized>(
    addr_multiget: AddressbookMultigetRequest,
    req: HttpRequest,
    user: &User,
    principal: &str,
    cal_id: &str,
    addr_store: &AS,
) -> Result<
    MultistatusElement<
        PropstatWrapper<EitherProp<AddressObjectProp, CommonPropertiesProp>>,
        String,
    >,
    Error,
> {
    let principal_url = PrincipalResource::get_url(req.resource_map(), vec![principal]).unwrap();
    let (objects, not_found) = get_objects_addressbook_multiget(
        &addr_multiget,
        &principal_url,
        principal,
        cal_id,
        addr_store,
    )
    .await?;

    let props = match addr_multiget.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            vec!["propname".to_owned()]
        }
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags.into_inner(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let mut responses = Vec::new();
    for object in objects {
        let path = format!("{}/{}", req.path(), object.get_id());
        responses.push(
            AddressObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind(&path, props.clone(), user, req.resource_map())?,
        );
    }

    let not_found_responses = not_found
        .into_iter()
        .map(|path| ResponseElement {
            href: path,
            status: Some(format!("HTTP/1.1 {}", StatusCode::NOT_FOUND)),
            ..Default::default()
        })
        .collect();

    Ok(MultistatusElement {
        responses,
        member_responses: not_found_responses,
        ..Default::default()
    })
}
