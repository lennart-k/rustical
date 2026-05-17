use crate::{
    Error, Principal,
    extensions::{CommonPropertiesExtension, CommonPropertiesProp},
    namespace::NS_DAV,
    privileges::UserPrivilegeSet,
    resource::{PrincipalUri, Resource},
    resources::RootResource,
    rfc_3986_percent_encode,
    xml::{Resourcetype, ResourcetypeInner},
};
use http::Uri;

#[test]
fn test_root_resource() {
    let resource = RootResource::<TestPrincipal, TestPrincipal>::default();
    let propfind = RootResource::<TestPrincipal, TestPrincipal>::parse_propfind(
        r#"<?xml version="1.0" encoding="UTF-8"?><propfind xmlns="DAV:"><allprop/></propfind>"#,
    )
    .unwrap();

    let _response = resource
        .propfind(
            "/",
            &propfind.prop,
            propfind.include.as_ref(),
            &TestPrincipalUri,
            &TestPrincipal("user".to_owned()),
        )
        .unwrap();
}

#[derive(Debug, Clone)]
pub struct TestPrincipal(pub String);

impl Principal for TestPrincipal {
    fn get_id(&self) -> &str {
        &self.0
    }
}

impl Resource for TestPrincipal {
    type Prop = CommonPropertiesProp;
    type Error = Error;
    type Principal = Self;

    fn is_collection(&self) -> bool {
        true
    }

    fn get_resourcetype(&self) -> crate::xml::Resourcetype {
        Resourcetype(&[ResourcetypeInner(Some(NS_DAV), "collection")])
    }

    fn get_prop(
        &self,
        principal_uri: &impl crate::resource::PrincipalUri,
        principal: &Self::Principal,
        prop: &<Self::Prop as rustical_xml::PropName>::Names,
    ) -> Result<Self::Prop, Self::Error> {
        <Self as CommonPropertiesExtension>::get_prop(self, principal_uri, principal, prop)
    }

    fn get_displayname(&self) -> Option<&str> {
        Some(&self.0)
    }

    fn get_user_privileges(
        &self,
        principal: &Self::Principal,
    ) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(
            principal.get_id() == self.get_id(),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct TestPrincipalUri;

impl PrincipalUri for TestPrincipalUri {
    fn principal_collection(&self) -> Uri {
        Uri::from_static("/")
    }
    fn principal_uri(&self, principal: &str) -> Uri {
        let principal = rfc_3986_percent_encode(principal);
        Uri::builder()
            .path_and_query(format!("/{principal}/"))
            .build()
            .unwrap()
    }
}
