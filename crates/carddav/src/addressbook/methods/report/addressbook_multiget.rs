use crate::{
    Error,
    address_object::resource::{
        AddressObjectPropWrapper, AddressObjectPropWrapperName, AddressObjectResource,
    },
};
use actix_web::{
    dev::{Path, ResourceDef},
    http::StatusCode,
};
use rustical_dav::{
    resource::{PrincipalUri, Resource},
    xml::{MultistatusElement, PropfindType, multistatus::ResponseElement},
};
use rustical_ical::AddressObject;
use rustical_store::{AddressbookStore, auth::User};
use rustical_xml::XmlDeserialize;

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[allow(dead_code)]
#[xml(ns = "rustical_dav::namespace::NS_DAV")]
pub struct AddressbookMultigetRequest {
    #[xml(ns = "rustical_dav::namespace::NS_DAV", ty = "untagged")]
    pub(crate) prop: PropfindType<AddressObjectPropWrapperName>,
    #[xml(ns = "rustical_dav::namespace::NS_DAV", flatten)]
    pub(crate) href: Vec<String>,
}

pub async fn get_objects_addressbook_multiget<AS: AddressbookStore>(
    addressbook_multiget: &AddressbookMultigetRequest,
    path: &str,
    principal: &str,
    addressbook_id: &str,
    store: &AS,
) -> Result<(Vec<AddressObject>, Vec<String>), Error> {
    let resource_def = ResourceDef::prefix(path).join(&ResourceDef::new("/{object_id}.vcf"));

    let mut result = vec![];
    let mut not_found = vec![];

    for href in &addressbook_multiget.href {
        let mut path = Path::new(href.as_str());
        if !resource_def.capture_match_info(&mut path) {
            not_found.push(href.to_owned());
            continue;
        };
        let object_id = path.get("object_id").unwrap();
        match store
            .get_object(principal, addressbook_id, object_id, false)
            .await
        {
            Ok(object) => result.push(object),
            Err(rustical_store::Error::NotFound) => not_found.push(href.to_owned()),
            // TODO: Maybe add error handling on a per-object basis
            Err(err) => return Err(err.into()),
        };
    }

    Ok((result, not_found))
}

pub async fn handle_addressbook_multiget<AS: AddressbookStore>(
    addr_multiget: &AddressbookMultigetRequest,
    prop: &PropfindType<AddressObjectPropWrapperName>,
    path: &str,
    puri: &impl PrincipalUri,
    user: &User,
    principal: &str,
    cal_id: &str,
    addr_store: &AS,
) -> Result<MultistatusElement<AddressObjectPropWrapper, String>, Error> {
    let (objects, not_found) =
        get_objects_addressbook_multiget(addr_multiget, path, principal, cal_id, addr_store)
            .await?;

    let mut responses = Vec::new();
    for object in objects {
        let path = format!("{}/{}.vcf", path, object.get_id());
        responses.push(
            AddressObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind_typed(&path, prop, puri, user)?,
        );
    }

    let not_found_responses = not_found
        .into_iter()
        .map(|path| ResponseElement {
            href: path,
            status: Some(StatusCode::NOT_FOUND),
            ..Default::default()
        })
        .collect();

    Ok(MultistatusElement {
        responses,
        member_responses: not_found_responses,
        ..Default::default()
    })
}
