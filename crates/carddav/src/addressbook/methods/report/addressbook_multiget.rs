use crate::{
    Error,
    address_object::{
        AddressObjectPropWrapper, AddressObjectPropWrapperName, resource::AddressObjectResource,
    },
};
use http::StatusCode;
use rustical_dav::{
    resource::{PrincipalUri, Resource},
    xml::{MultistatusElement, PropfindType, multistatus::ResponseElement},
};
use rustical_ical::AddressObject;
use rustical_store::{AddressbookStore, auth::Principal};
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
    let mut result = vec![];
    let mut not_found = vec![];

    for href in &addressbook_multiget.href {
        if let Some(filename) = href.strip_prefix(path) {
            let filename = filename.trim_start_matches("/");
            if let Some(object_id) = filename.strip_suffix(".vcf") {
                match store
                    .get_object(principal, addressbook_id, object_id, false)
                    .await
                {
                    Ok(object) => result.push(object),
                    Err(rustical_store::Error::NotFound) => not_found.push(href.to_owned()),
                    Err(err) => return Err(err.into()),
                };
            } else {
                not_found.push(href.to_owned());
                continue;
            }
        } else {
            not_found.push(href.to_owned());
            continue;
        }
    }

    Ok((result, not_found))
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_addressbook_multiget<AS: AddressbookStore>(
    addr_multiget: &AddressbookMultigetRequest,
    prop: &PropfindType<AddressObjectPropWrapperName>,
    path: &str,
    puri: &impl PrincipalUri,
    user: &Principal,
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
            .propfind(&path, prop, puri, user)?,
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
