use super::prop::SupportedAddressData;
use crate::Error;
use crate::addressbook::prop::{
    AddressbookProp, AddressbookPropName, AddressbookPropWrapper, AddressbookPropWrapperName,
};
use derive_more::derive::{From, Into};
use rustical_dav::extensions::{CommonPropertiesExtension, SyncTokenExtension};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{PrincipalUri, Resource, ResourceName};
use rustical_dav::xml::{Resourcetype, ResourcetypeInner, SupportedReportSet};
use rustical_dav_push::DavPushExtension;
use rustical_store::Addressbook;
use rustical_store::auth::Principal;
use std::borrow::Cow;

#[derive(Clone, Debug, From, Into)]
pub struct AddressbookResource(pub(crate) Addressbook);

impl ResourceName for AddressbookResource {
    fn get_name(&self) -> Cow<'_, str> {
        Cow::from(&self.0.id)
    }
}

impl SyncTokenExtension for AddressbookResource {
    fn get_synctoken(&self) -> String {
        self.0.format_synctoken()
    }
}

impl DavPushExtension for AddressbookResource {
    fn get_topic(&self) -> String {
        self.0.push_topic.clone()
    }
}

impl Resource for AddressbookResource {
    type Prop = AddressbookPropWrapper;
    type Error = Error;
    type Principal = Principal;

    fn is_collection(&self) -> bool {
        true
    }

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "collection"),
            ResourcetypeInner(Some(rustical_dav::namespace::NS_CARDDAV), "addressbook"),
        ])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &Principal,
        prop: &AddressbookPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            AddressbookPropWrapperName::Addressbook(prop) => {
                AddressbookPropWrapper::Addressbook(match prop {
                    AddressbookPropName::MaxResourceSize => {
                        AddressbookProp::MaxResourceSize(10_000_000)
                    }
                    AddressbookPropName::SupportedReportSet => {
                        AddressbookProp::SupportedReportSet(SupportedReportSet::all())
                    }
                    AddressbookPropName::AddressbookDescription => {
                        AddressbookProp::AddressbookDescription(self.0.description.clone())
                    }
                    AddressbookPropName::SupportedAddressData => {
                        AddressbookProp::SupportedAddressData(SupportedAddressData::default())
                    }
                })
            }

            AddressbookPropWrapperName::SyncToken(prop) => AddressbookPropWrapper::SyncToken(
                <Self as SyncTokenExtension>::get_prop(self, prop)?,
            ),
            AddressbookPropWrapperName::DavPush(prop) => {
                AddressbookPropWrapper::DavPush(<Self as DavPushExtension>::get_prop(self, prop)?)
            }
            AddressbookPropWrapperName::Common(prop) => AddressbookPropWrapper::Common(
                CommonPropertiesExtension::get_prop(self, puri, user, prop)?,
            ),
        })
    }

    fn set_prop(&mut self, prop: Self::Prop) -> Result<(), rustical_dav::Error> {
        match prop {
            AddressbookPropWrapper::Addressbook(prop) => match prop {
                AddressbookProp::AddressbookDescription(description) => {
                    self.0.description = description;
                    Ok(())
                }
                AddressbookProp::MaxResourceSize(_)
                | AddressbookProp::SupportedReportSet(_)
                | AddressbookProp::SupportedAddressData(_) => {
                    Err(rustical_dav::Error::PropReadOnly)
                }
            },
            AddressbookPropWrapper::SyncToken(prop) => SyncTokenExtension::set_prop(self, prop),
            AddressbookPropWrapper::DavPush(prop) => DavPushExtension::set_prop(self, prop),
            AddressbookPropWrapper::Common(prop) => CommonPropertiesExtension::set_prop(self, prop),
        }
    }

    fn remove_prop(
        &mut self,
        prop: &AddressbookPropWrapperName,
    ) -> Result<(), rustical_dav::Error> {
        match prop {
            AddressbookPropWrapperName::Addressbook(prop) => match prop {
                AddressbookPropName::AddressbookDescription => {
                    self.0.description = None;
                    Ok(())
                }
                AddressbookPropName::MaxResourceSize
                | AddressbookPropName::SupportedReportSet
                | AddressbookPropName::SupportedAddressData => {
                    Err(rustical_dav::Error::PropReadOnly)
                }
            },
            AddressbookPropWrapperName::SyncToken(prop) => {
                SyncTokenExtension::remove_prop(self, prop)
            }
            AddressbookPropWrapperName::DavPush(prop) => DavPushExtension::remove_prop(self, prop),
            AddressbookPropWrapperName::Common(prop) => {
                CommonPropertiesExtension::remove_prop(self, prop)
            }
        }
    }

    fn get_displayname(&self) -> Option<&str> {
        self.0.displayname.as_deref()
    }
    fn set_displayname(&mut self, name: Option<String>) -> Result<(), rustical_dav::Error> {
        self.0.displayname = name;
        Ok(())
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.0.principal)
    }

    fn get_user_privileges(&self, user: &Principal) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(
            user.is_principal(&self.0.principal),
        ))
    }
}
