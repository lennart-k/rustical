pub mod root;

pub use root::{RootResource, RootResourceService};

#[cfg(test)]
pub mod test {
    use crate::{
        Error, Principal,
        extensions::{CommonPropertiesExtension, CommonPropertiesProp},
        namespace::NS_DAV,
        privileges::UserPrivilegeSet,
        resource::{PrincipalUri, Resource},
        xml::{Resourcetype, ResourcetypeInner},
    };

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
        fn principal_collection(&self) -> String {
            "/".to_owned()
        }
        fn principal_uri(&self, principal: &str) -> String {
            format!("/{principal}/")
        }
    }
}
