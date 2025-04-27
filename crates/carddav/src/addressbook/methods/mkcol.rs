use crate::Error;
use actix_web::web::Path;
use actix_web::{HttpResponse, web::Data};
use rustical_store::{Addressbook, AddressbookStore, auth::User};
use rustical_xml::{XmlDeserialize, XmlDocument, XmlRootTag};
use tracing::instrument;
use tracing_actix_web::RootSpan;

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub struct Resourcetype {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    addressbook: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    collection: Option<()>,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub struct MkcolAddressbookProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    resourcetype: Option<Resourcetype>,
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    displayname: Option<String>,
    #[xml(rename = b"addressbook-description")]
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    description: Option<String>,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub struct PropElement<T: XmlDeserialize> {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    prop: T,
}

#[derive(XmlDeserialize, XmlRootTag, Clone, Debug, PartialEq)]
#[xml(root = b"mkcol")]
#[xml(ns = "rustical_dav::namespace::NS_DAV")]
struct MkcolRequest {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    set: PropElement<MkcolAddressbookProp>,
}

#[instrument(parent = root_span.id(), skip(store, root_span))]
pub async fn route_mkcol<AS: AddressbookStore>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    store: Data<AS>,
    root_span: RootSpan,
) -> Result<HttpResponse, Error> {
    let (principal, addressbook_id) = path.into_inner();
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let request = MkcolRequest::parse_str(&body)?;
    let request = request.set.prop;

    let addressbook = Addressbook {
        id: addressbook_id.to_owned(),
        principal: principal.to_owned(),
        displayname: request.displayname,
        description: request.description,
        deleted_at: None,
        synctoken: 0,
        push_topic: uuid::Uuid::new_v4().to_string(),
    };

    match store
        .get_addressbook(&principal, &addressbook_id, true)
        .await
    {
        Err(rustical_store::Error::NotFound) => {
            // No conflict, no worries
        }
        Ok(_) => {
            // oh no, there's a conflict
            return Ok(HttpResponse::Conflict().body("An addressbook already exists at this URI"));
        }
        Err(err) => {
            // some other error
            return Err(err.into());
        }
    }

    match store.insert_addressbook(addressbook).await {
        // TODO: The spec says we should return a mkcol-response.
        // However, it works without one but breaks on iPadOS when using an empty one :)
        Ok(()) => Ok(HttpResponse::Created()
            .insert_header(("Cache-Control", "no-cache"))
            .body("")),
        Err(err) => {
            dbg!(err.to_string());
            Err(err.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_mkcol() {
        let mkcol_request = MkcolRequest::parse_str(r#"
            <?xml version='1.0' encoding='UTF-8' ?>
            <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
                <set>
                    <prop>
                        <resourcetype>
                            <collection />
                            <CARD:addressbook />
                        </resourcetype>
                        <displayname>whoops</displayname>
                        <CARD:addressbook-description>okay</CARD:addressbook-description>
                    </prop>
                </set>
            </mkcol>
        "#).unwrap();
        assert_eq!(
            mkcol_request,
            MkcolRequest {
                set: PropElement {
                    prop: MkcolAddressbookProp {
                        resourcetype: Some(Resourcetype {
                            addressbook: Some(()),
                            collection: Some(())
                        }),
                        displayname: Some("whoops".to_owned()),
                        description: Some("okay".to_owned())
                    }
                }
            }
        )
    }
}
