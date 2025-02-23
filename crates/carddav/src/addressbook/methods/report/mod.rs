use crate::Error;
use actix_web::{
    web::{Data, Path},
    HttpRequest, Responder,
};
use addressbook_multiget::{handle_addressbook_multiget, AddressbookMultigetRequest};
use rustical_dav::xml::sync_collection::SyncCollectionRequest;
use rustical_store::{auth::User, AddressbookStore};
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
    SyncCollection(SyncCollectionRequest),
}

#[instrument(skip(req, addr_store))]
pub async fn route_report_addressbook<AS: AddressbookStore>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    req: HttpRequest,
    addr_store: Data<AS>,
) -> Result<impl Responder, Error> {
    let (principal, addressbook_id) = path.into_inner();
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let request = ReportRequest::parse_str(&body)?;

    Ok(match request.clone() {
        ReportRequest::AddressbookMultiget(addr_multiget) => {
            handle_addressbook_multiget(
                addr_multiget,
                req,
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
                req,
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
    use rustical_dav::xml::{sync_collection::SyncLevel, PropElement, Propname};

    use super::*;

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
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(vec![Propname(
                    "getetag".to_owned()
                )])),
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
                    Propname("getetag".to_owned()),
                    Propname("address-data".to_owned())
                ])),
                href: vec![
                    "/carddav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b".to_owned()
                ]
            })
        )
    }
}
