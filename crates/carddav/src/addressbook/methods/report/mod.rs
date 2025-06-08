use crate::{
    CardDavPrincipalUri, Error, address_object::resource::AddressObjectPropWrapperName,
    addressbook::resource::AddressbookResourceService,
};
use addressbook_multiget::{AddressbookMultigetRequest, handle_addressbook_multiget};
use axum::{
    Extension,
    extract::{OriginalUri, Path, State},
    response::IntoResponse,
};
use rustical_dav::xml::{PropfindType, sync_collection::SyncCollectionRequest};
use rustical_store::{AddressbookStore, SubscriptionStore, auth::User};
use rustical_xml::{XmlDeserialize, XmlDocument};
use sync_collection::handle_sync_collection;
use tracing::instrument;

mod addressbook_multiget;
mod sync_collection;

#[derive(XmlDeserialize, XmlDocument, Clone, Debug, PartialEq)]
pub(crate) enum ReportRequest {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookMultiget(AddressbookMultigetRequest),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    SyncCollection(SyncCollectionRequest<AddressObjectPropWrapperName>),
}

impl ReportRequest {
    fn props(&self) -> &PropfindType<AddressObjectPropWrapperName> {
        match self {
            ReportRequest::AddressbookMultiget(AddressbookMultigetRequest { prop, .. }) => prop,
            ReportRequest::SyncCollection(SyncCollectionRequest { prop, .. }) => prop,
        }
    }
}

#[instrument(skip(addr_store))]
pub async fn route_report_addressbook<AS: AddressbookStore, S: SubscriptionStore>(
    Path((principal, addressbook_id)): Path<(String, String)>,
    user: User,
    OriginalUri(uri): OriginalUri,
    Extension(puri): Extension<CardDavPrincipalUri>,
    State(AddressbookResourceService { addr_store, .. }): State<AddressbookResourceService<AS, S>>,
    body: String,
) -> Result<impl IntoResponse, Error> {
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let request = ReportRequest::parse_str(&body)?;

    Ok(match &request {
        ReportRequest::AddressbookMultiget(addr_multiget) => {
            handle_addressbook_multiget(
                addr_multiget,
                request.props(),
                uri.path(),
                &puri,
                &user,
                &principal,
                &addressbook_id,
                addr_store.as_ref(),
            )
            .await?
        }
        ReportRequest::SyncCollection(sync_collection) => {
            handle_sync_collection(
                sync_collection,
                uri.path(),
                &puri,
                &user,
                &principal,
                &addressbook_id,
                addr_store.as_ref(),
            )
            .await?
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address_object::resource::AddressObjectPropName;
    use rustical_dav::xml::{PropElement, sync_collection::SyncLevel};

    #[test]
    fn test_xml_sync_collection() {
        let report_request = ReportRequest::parse_str(
            r#"
        <?xml version='1.0' encoding='UTF-8' ?>
        <sync-collection xmlns="DAV:">
            <sync-token />
            <sync-level>1</sync-level>
            <prop>
                <getetag />
            </prop>
        </sync-collection>"#,
        )
        .unwrap();
        assert_eq!(
            report_request,
            ReportRequest::SyncCollection(SyncCollectionRequest {
                sync_token: "".to_owned(),
                sync_level: SyncLevel::One,
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(
                    vec![AddressObjectPropWrapperName::AddressObject(
                        AddressObjectPropName::Getetag
                    )],
                    vec![]
                )),
                limit: None
            })
        )
    }

    #[test]
    fn test_xml_addressbook_multiget() {
        let report_request = ReportRequest::parse_str(r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <addressbook-multiget xmlns="urn:ietf:params:xml:ns:carddav" xmlns:D="DAV:">
                <D:prop>
                    <D:getetag/>
                    <address-data/>
                </D:prop>
                <D:href>/carddav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b</D:href>
            </addressbook-multiget>
        "#).unwrap();

        assert_eq!(
            report_request,
            ReportRequest::AddressbookMultiget(AddressbookMultigetRequest {
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(vec![
                        AddressObjectPropWrapperName::AddressObject(
                        AddressObjectPropName::Getetag
                    ),
                        AddressObjectPropWrapperName::AddressObject(
                        AddressObjectPropName::AddressData
                    ),
                ], vec![])),
                href: vec![
                    "/carddav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b".to_owned()
                ]
            })
        )
    }
}
