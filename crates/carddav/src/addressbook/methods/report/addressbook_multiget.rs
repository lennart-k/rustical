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

#[cfg(test)]
mod tests {
    use crate::addressbook::methods::report::addressbook_multiget::{
        AddressbookMultigetRequest, get_objects_addressbook_multiget,
    };
    use http::Uri;
    use rustical_ical::AddressObject;
    use rustical_store::AddressbookStore;

    struct MockAddrStore {
        principal: &'static str,
        addr_id: &'static str,
    }

    #[async_trait::async_trait]
    impl AddressbookStore for MockAddrStore {
        async fn get_addressbook(
            &self,
            _principal: &str,
            _id: &str,
            _show_deleted: bool,
        ) -> Result<rustical_store::Addressbook, rustical_store::Error> {
            panic!()
        }

        async fn sync_changes(
            &self,
            _principal: &str,
            _addressbook_id: &str,
            _synctoken: i64,
        ) -> Result<
            (
                Vec<(String, rustical_ical::AddressObject)>,
                Vec<String>,
                i64,
            ),
            rustical_store::Error,
        > {
            panic!()
        }

        async fn get_addressbooks(
            &self,
            _principal: &str,
        ) -> Result<Vec<rustical_store::Addressbook>, rustical_store::Error> {
            panic!()
        }

        async fn get_deleted_addressbooks(
            &self,
            _principal: &str,
        ) -> Result<Vec<rustical_store::Addressbook>, rustical_store::Error> {
            panic!()
        }

        async fn update_addressbook(
            &self,
            _principal: &str,
            _id: &str,
            _addressbook: rustical_store::Addressbook,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn insert_addressbook(
            &self,
            _addressbook: rustical_store::Addressbook,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn delete_addressbook(
            &self,
            _principal: &str,
            _name: &str,
            _use_trashbin: bool,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn restore_addressbook(
            &self,
            _principal: &str,
            _name: &str,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn addressbook_metadata(
            &self,
            _principal: &str,
            _addressbook_id: &str,
        ) -> Result<rustical_store::CollectionMetadata, rustical_store::Error> {
            panic!()
        }

        async fn get_objects(
            &self,
            _principal: &str,
            _addressbook_id: &str,
        ) -> Result<Vec<(String, rustical_ical::AddressObject)>, rustical_store::Error> {
            panic!()
        }

        async fn put_object(
            &self,
            _principal: &str,
            _addressbook_id: &str,
            _object_id: &str,
            _object: rustical_ical::AddressObject,
            _overwrite: bool,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn delete_object(
            &self,
            _principal: &str,
            _addressbook_id: &str,
            _object_id: &str,
            _use_trashbin: bool,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn restore_object(
            &self,
            _principal: &str,
            _addressbook_id: &str,
            _object_id: &str,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn import_addressbook(
            &self,
            _addressbook: rustical_store::Addressbook,
            _objects: Vec<(String, rustical_ical::AddressObject)>,
            _merge_existing: bool,
        ) -> Result<(), rustical_store::Error> {
            panic!()
        }

        async fn get_object(
            &self,
            principal: &str,
            addressbook_id: &str,
            _object_id: &str,
            _show_deleted: bool,
        ) -> Result<rustical_ical::AddressObject, rustical_store::Error> {
            if self.principal != principal || self.addr_id != addressbook_id {
                return Err(rustical_store::Error::NotFound);
            }
            Ok(AddressObject::from_vcf(
                r"BEGIN:VCARD
VERSION:4.0
N:Mustermann;Erika;;Dr.;
FN:Dr. Erika Mustermann
REV:20140301T221110Z
END:VCARD"
                    .to_string(),
            )
            .unwrap())
        }
    }

    #[tokio::test]
    #[rstest::rstest]
    #[case("/carddav/principal/user%40example%2Ecom/contacts/")]
    #[case("/carddav/principal/user%40example.com/contacts/")]
    async fn test_multiget_url_escaping(#[case] request_path: &'static str) {
        let req = AddressbookMultigetRequest {
            prop: rustical_dav::xml::PropfindType::Propname,
            href: vec![
                "/carddav/principal/user%40example%2Ecom/contacts/hello.vcf".to_string(),
                "/carddav/principal/user@example.com/contacts/unescaped.vcf".to_string(),
                "/carddav/principal/user%40example.com/contacts/shouldwork.vcf".to_string(),
                "/carddav/principal/user%40example.com/contacts/notfound".to_string(),
                "/carddav/principal/user%40example%2Ecom/wrongcontacts/hello.vcf".to_string(),
                "asd asd".to_string(),
                "/carddav/principal/user%40example.com/contacts".to_string(),
            ],
        };

        let (result, not_found) = get_objects_addressbook_multiget(
            &req,
            &Uri::from_static(request_path),
            "user@example.com",
            "contacts",
            &MockAddrStore {
                principal: "user@example.com",
                addr_id: "contacts",
            },
        )
        .await
        .unwrap();

        let found: Vec<String> = result.into_iter().map(|(href, _)| href).collect();
        similar_asserts::assert_eq!(
            found,
            vec![
                "hello".to_string(),
                "unescaped".to_string(),
                "shouldwork".to_string()
            ]
        );

        similar_asserts::assert_eq!(
            not_found,
            vec![
                "/carddav/principal/user%40example.com/contacts/notfound".to_string(),
                "/carddav/principal/user%40example%2Ecom/wrongcontacts/hello.vcf".to_string(),
                "asd asd".to_string(),
                "/carddav/principal/user%40example.com/contacts".to_string(),
            ]
        );
    }
}
