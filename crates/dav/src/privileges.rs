use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "kebab-case")]
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

impl Serialize for UserPrivilegeSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        #[serde(rename_all = "kebab-case")]
        pub struct UserPrivilegeWrapper<'a> {
            #[serde(rename = "$value")]
            privilege: &'a UserPrivilege,
        }
        #[derive(Serialize)]
        #[serde(rename_all = "kebab-case")]
        pub struct FakeUserPrivilegeSet<'a> {
            #[serde(rename = "privilege")]
            privileges: Vec<UserPrivilegeWrapper<'a>>,
        }
        FakeUserPrivilegeSet {
            privileges: self
                .privileges
                .iter()
                .map(|privilege| UserPrivilegeWrapper { privilege })
                .collect(),
        }
        .serialize(serializer)
    }
}

// TODO: implement Deserialize once we need it
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
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
