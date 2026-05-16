use std::str::FromStr;

use crate::{
    Error,
    address_object::{
        AddressObjectPropWrapper, AddressObjectPropWrapperName, resource::AddressObjectResource,
    },
};
use http::{StatusCode, Uri};
use rustical_dav::{
    resolve_child_uri,
    resource::{PrincipalUri, Resource},
    xml::{MultistatusElement, PropfindType, multistatus::ResponseElement},
};
use rustical_ical::AddressObject;
use rustical_store::{AddressbookStore, auth::Principal};
use rustical_xml::XmlDeserialize;

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
#[xml(ns = "rustical_dav::namespace::NS_DAV")]
pub struct AddressbookMultigetRequest {
    #[xml(ns = "rustical_dav::namespace::NS_DAV", ty = "untagged")]
    pub(crate) prop: PropfindType<AddressObjectPropWrapperName>,
    #[xml(ns = "rustical_dav::namespace::NS_DAV", flatten)]
    pub(crate) href: Vec<String>,
}

pub async fn get_objects_addressbook_multiget<AS: AddressbookStore>(
    request: &AddressbookMultigetRequest,
    collection_uri: &Uri,
    principal: &str,
    addressbook_id: &str,
    store: &AS,
) -> Result<(Vec<(String, AddressObject)>, Vec<String>), Error> {
    let mut result = vec![];
    let mut not_found = vec![];

    for href in &request.href {
        let Ok(child_uri) = Uri::from_str(href) else {
            not_found.push(href.clone());
            continue;
        };
        let Some(subpath) = resolve_child_uri(collection_uri, &child_uri) else {
            not_found.push(href.clone());
            continue;
        };
        let [filename] = subpath.as_slice() else {
            not_found.push(href.clone());
            continue;
        };
        let Some(object_id) = filename.strip_suffix(".vcf") else {
            not_found.push(href.clone());
            continue;
        };
        match store
            .get_object(principal, addressbook_id, object_id, false)
            .await
        {
            Ok(object) => result.push((object_id.to_owned(), object)),
            Err(rustical_store::Error::NotFound) => not_found.push(href.clone()),
            Err(err) => return Err(err.into()),
        }
    }

    Ok((result, not_found))
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_addressbook_multiget<AS: AddressbookStore>(
    addr_multiget: &AddressbookMultigetRequest,
    prop: &PropfindType<AddressObjectPropWrapperName>,
    collection_uri: &Uri,
    puri: &impl PrincipalUri,
    user: &Principal,
    principal: &str,
    cal_id: &str,
    addr_store: &AS,
) -> Result<MultistatusElement<AddressObjectPropWrapper, String>, Error> {
    let (objects, not_found) = get_objects_addressbook_multiget(
        addr_multiget,
        collection_uri,
        principal,
        cal_id,
        addr_store,
    )
    .await?;

    let mut responses = Vec::new();
    for (object_id, object) in objects {
        let path = format!(
            "{path}/{object_id}.vcf",
            path = collection_uri.path().trim_end_matches('/')
        );
        responses.push(
            AddressObjectResource {
                object,
                object_id,
                principal: principal.to_owned(),
            }
            .propfind(&path, prop, None, puri, user)?,
        );
    }

    let not_found_responses = not_found
        .into_iter()
        .map(|path| ResponseElement {
            href: Uri::from_str(&path).unwrap(),
            status: Some(StatusCode::NOT_FOUND),
            propstat: vec![],
        })
        .collect();

    Ok(MultistatusElement {
        responses,
        member_responses: not_found_responses,
        ..Default::default()
    })
}
