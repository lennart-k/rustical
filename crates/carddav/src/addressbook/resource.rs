use super::prop::SupportedAddressData;
use crate::Error;
use crate::addressbook::prop::{
    AddressbookProp, AddressbookPropName, AddressbookPropWrapper, AddressbookPropWrapperName,
    SupportedCollationSet,
};
use rustical_dav::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, CommonPropertiesPropName, SyncTokenExtension,
};
use rustical_dav::namespace::{NS_CARDDAV, NS_DAV};
use rustical_dav::privileges::{UserPrivilege, UserPrivilegeSet};
use rustical_dav::resource::{PrincipalUri, Resource, ResourceName};
use rustical_dav::resourcetype;
use rustical_dav::xml::{Resourcetype, SupportedReportSet};
use rustical_dav_push::{DavPushExtension, VapidPublicKeyB64};
use rustical_store::access::{self, Resolved, ResolvedKind};
use rustical_store::auth::Principal;
use rustical_store::{Access, Addressbook};
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct AddressbookResource {
    pub(crate) addressbook: Addressbook,
    pub(crate) vapid_pubkey: VapidPublicKeyB64,
    /// The home of the path
    pub(crate) home: String,
    /// The name of the addressbook in the home (for a share not the owner's id)
    pub(crate) name: String,
    /// What the addressbook is: the owner, the id and whether the home owns it or it is shared
    pub(crate) resolved: Resolved,
}

impl ResourceName for AddressbookResource {
    fn get_name(&self) -> Cow<'_, str> {
        Cow::from(&self.name)
    }
}

impl SyncTokenExtension for AddressbookResource {
    fn get_synctoken(&self) -> String {
        self.addressbook.format_synctoken()
    }
}

impl DavPushExtension for AddressbookResource {
    fn get_topic(&self) -> String {
        self.addressbook.push_topic.clone()
    }

    fn get_vapid_public_key(&self) -> Option<VapidPublicKeyB64> {
        Some(self.vapid_pubkey.clone())
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
        resourcetype!((NS_DAV, "collection"), (NS_CARDDAV, "addressbook"))
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
                    AddressbookPropName::SupportedCollationSet => {
                        AddressbookProp::SupportedCollationSet(SupportedCollationSet::default())
                    }
                    AddressbookPropName::SupportedReportSet => {
                        AddressbookProp::SupportedReportSet(SupportedReportSet::all())
                    }
                    AddressbookPropName::AddressbookDescription => {
                        AddressbookProp::AddressbookDescription(
                            self.addressbook.description.clone(),
                        )
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
        // A sharee changes only its own name for the share
        if !self.resolved.is_own()
            && !matches!(
                prop,
                AddressbookPropWrapper::Common(CommonPropertiesProp::Displayname(_))
            )
        {
            return Err(rustical_dav::Error::PropReadOnly);
        }
        match prop {
            AddressbookPropWrapper::Addressbook(prop) => match prop {
                AddressbookProp::AddressbookDescription(description) => {
                    self.addressbook.description = description;
                    Ok(())
                }
                AddressbookProp::MaxResourceSize(_)
                | AddressbookProp::SupportedCollationSet(_)
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
        if !self.resolved.is_own()
            && !matches!(
                prop,
                AddressbookPropWrapperName::Common(CommonPropertiesPropName::Displayname)
            )
        {
            return Err(rustical_dav::Error::PropReadOnly);
        }
        match prop {
            AddressbookPropWrapperName::Addressbook(prop) => match prop {
                AddressbookPropName::AddressbookDescription => {
                    self.addressbook.description = None;
                    Ok(())
                }
                AddressbookPropName::MaxResourceSize
                | AddressbookPropName::SupportedCollationSet
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
        self.addressbook.displayname.as_deref()
    }
    fn set_displayname(&mut self, name: Option<String>) -> Result<(), rustical_dav::Error> {
        self.addressbook.displayname = name;
        Ok(())
    }

    fn get_owner(&self) -> Option<&str> {
        Some(self.resolved.owner())
    }

    fn get_user_privileges(&self, user: &Principal) -> Result<UserPrivilegeSet, Self::Error> {
        let may_use = access::may_use_home(user, &self.home);
        Ok(match self.resolved.kind() {
            ResolvedKind::Own => UserPrivilegeSet::owner_only(may_use),
            ResolvedKind::Copy { .. } if !may_use => UserPrivilegeSet::default(),
            ResolvedKind::Copy {
                access: Access::Read,
                ..
            } => UserPrivilegeSet::from([
                UserPrivilege::Read,
                UserPrivilege::ReadCurrentUserPrivilegeSet,
                // The sharee's own name
                UserPrivilege::WriteProperties,
            ]),
            ResolvedKind::Copy {
                access: Access::Write,
                ..
            } => UserPrivilegeSet::from([
                UserPrivilege::Read,
                UserPrivilege::ReadCurrentUserPrivilegeSet,
                UserPrivilege::WriteProperties,
                UserPrivilege::Write,
                UserPrivilege::WriteContent,
            ]),
        })
    }
}
