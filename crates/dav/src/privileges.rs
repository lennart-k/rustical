use rustical_xml::{XmlDeserialize, XmlSerialize};
use std::collections::HashSet;

#[derive(Debug, Clone, XmlSerialize, XmlDeserialize, Eq, Hash, PartialEq)]
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
    fn serialize<W: std::io::Write>(
        &self,
        ns: Option<&[u8]>,
        tag: Option<&[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()> {
        #[derive(XmlSerialize)]
        pub struct FakeUserPrivilegeSet {
            #[xml(rename = b"privilege", flatten)]
            privileges: Vec<UserPrivilege>,
        }

        FakeUserPrivilegeSet {
            privileges: self.privileges.iter().cloned().collect(),
        }
        .serialize(ns, tag, writer)
    }

    #[allow(refining_impl_trait)]
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
}

impl<const N: usize> From<[UserPrivilege; N]> for UserPrivilegeSet {
    fn from(privileges: [UserPrivilege; N]) -> Self {
        Self {
            privileges: HashSet::from(privileges),
        }
    }
}
