use itertools::Itertools;
use quick_xml::name::Namespace;
use rustical_xml::{XmlDeserialize, XmlSerialize};
use std::collections::{HashMap, HashSet};

// https://datatracker.ietf.org/doc/html/rfc3744
#[derive(Debug, Clone, XmlSerialize, XmlDeserialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum UserPrivilege {
    Read,
    Write,
    WriteProperties,
    WriteContent,
    ReadAcl,
    ReadCurrentUserPrivilegeSet,
    WriteAcl,
    All,
}

impl XmlSerialize for UserPrivilegeSet {
    fn serialize(
        &self,
        ns: Option<Namespace>,
        tag: Option<&[u8]>,
        namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut quick_xml::Writer<&mut Vec<u8>>,
    ) -> std::io::Result<()> {
        #[derive(XmlSerialize)]
        pub struct FakeUserPrivilegeSet {
            #[xml(rename = b"privilege", flatten)]
            privileges: Vec<UserPrivilege>,
        }

        FakeUserPrivilegeSet {
            privileges: self.privileges.iter().cloned().sorted().collect(),
        }
        .serialize(ns, tag, namespaces, writer)
    }

    fn attributes<'a>(&self) -> Option<Vec<quick_xml::events::attributes::Attribute<'a>>> {
        None
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserPrivilegeSet {
    privileges: HashSet<UserPrivilege>,
}

impl UserPrivilegeSet {
    pub fn has(&self, privilege: &UserPrivilege) -> bool {
        if (privilege == &UserPrivilege::WriteProperties
            || privilege == &UserPrivilege::WriteContent)
            && self.privileges.contains(&UserPrivilege::Write)
        {
            return true;
        }
        self.privileges.contains(privilege) || self.privileges.contains(&UserPrivilege::All)
    }

    pub fn all() -> Self {
        Self {
            privileges: HashSet::from([UserPrivilege::All]),
        }
    }

    pub fn owner_only(is_owner: bool) -> Self {
        if is_owner {
            Self::all()
        } else {
            Self::default()
        }
    }

    pub fn owner_read(is_owner: bool) -> Self {
        if is_owner {
            Self::read_only()
        } else {
            Self::default()
        }
    }

    pub fn owner_write_properties(is_owner: bool) -> Self {
        // Content is read-only but we can write properties
        if is_owner {
            Self::write_properties()
        } else {
            Self::default()
        }
    }

    pub fn read_only() -> Self {
        Self {
            privileges: HashSet::from([
                UserPrivilege::Read,
                UserPrivilege::ReadAcl,
                UserPrivilege::ReadCurrentUserPrivilegeSet,
            ]),
        }
    }

    pub fn write_properties() -> Self {
        Self {
            privileges: HashSet::from([
                UserPrivilege::Read,
                UserPrivilege::WriteProperties,
                UserPrivilege::ReadAcl,
                UserPrivilege::ReadCurrentUserPrivilegeSet,
            ]),
        }
    }
}

impl<const N: usize> From<[UserPrivilege; N]> for UserPrivilegeSet {
    fn from(privileges: [UserPrivilege; N]) -> Self {
        Self {
            privileges: HashSet::from(privileges),
        }
    }
}
