//! Sharing collections with other principals.
//!
//! A share gives one principal (the sharee, a user or a group) access to a calendar or an
//! addressbook of another principal (the owner). The shared collection appears in the
//! sharee's home under its own name, with the sharee's own display name and color.
use crate::Error;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::Serialize;
use std::str::FromStr;

/// What a share allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Access {
    /// Only read the collection.
    Read,
    /// Read, create, change and delete objects.
    Write,
}

impl Access {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

impl FromStr for Access {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            _ => Err(Error::Other(anyhow::anyhow!(
                "invalid share access {value:?}"
            ))),
        }
    }
}

/// Why the store refused to create a share.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ShareRejection {
    #[error("the owner cannot share a collection with themselves")]
    Owner,
    #[error("members of the owning group already have full access")]
    OwnerMember,
    #[error("no principal with this name or e-mail address")]
    UnknownSharee,
    #[error("this name and e-mail address belong to different principals")]
    AmbiguousSharee,
    #[error("birthday calendars and subscriptions cannot be shared")]
    NotShareable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CalendarShare {
    pub owner: String,
    pub cal_id: String,
    pub sharee: String,
    /// The id of the shared calendar in the sharee's home
    pub name: String,
    pub access: Access,
    pub displayname: Option<String>,
    pub color: Option<String>,
    pub order: Option<i64>,
    pub created_at: NaiveDateTime,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AddressbookShare {
    pub owner: String,
    pub addressbook_id: String,
    pub sharee: String,
    /// The id of the shared addressbook in the sharee's home
    pub name: String,
    pub access: Access,
    pub displayname: Option<String>,
    pub color: Option<String>,
    pub created_at: NaiveDateTime,
    pub created_by: Option<String>,
}

/// A change to the sharee's view of a share. `None` leaves a value unchanged,
/// `Some(None)` falls back to the owner's value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShareViewUpdate {
    pub displayname: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub order: Option<Option<i64>>,
}

impl ShareViewUpdate {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.displayname.is_none() && self.color.is_none() && self.order.is_none()
    }
}

/// The store of shares and of the principal data sharing needs (e-mail addresses).
#[async_trait]
pub trait ShareStore: Send + Sync + 'static {
    /// Shares a calendar with `sharee` (a principal id or an e-mail address).
    /// Sharing again with the same principal changes only the access.
    async fn share_calendar(
        &self,
        owner: &str,
        cal_id: &str,
        sharee: &str,
        access: Access,
        created_by: Option<&str>,
    ) -> Result<CalendarShare, Error>;
    async fn unshare_calendar(&self, owner: &str, cal_id: &str, sharee: &str) -> Result<(), Error>;
    async fn list_calendar_shares(
        &self,
        owner: &str,
        cal_id: &str,
    ) -> Result<Vec<CalendarShare>, Error>;
    /// The shares in the home of `sharee` whose calendar is not in the trash bin
    async fn calendars_shared_with(&self, sharee: &str) -> Result<Vec<CalendarShare>, Error>;
    /// The share at `(sharee, name)` if its calendar is not in the trash bin
    async fn get_calendar_share(
        &self,
        sharee: &str,
        name: &str,
    ) -> Result<Option<CalendarShare>, Error>;
    async fn update_calendar_share_view(
        &self,
        sharee: &str,
        name: &str,
        update: ShareViewUpdate,
    ) -> Result<(), Error>;

    /// Shares an addressbook with `sharee` (a principal id or an e-mail address).
    /// Sharing again with the same principal changes only the access.
    async fn share_addressbook(
        &self,
        owner: &str,
        addressbook_id: &str,
        sharee: &str,
        access: Access,
        created_by: Option<&str>,
    ) -> Result<AddressbookShare, Error>;
    async fn unshare_addressbook(
        &self,
        owner: &str,
        addressbook_id: &str,
        sharee: &str,
    ) -> Result<(), Error>;
    async fn list_addressbook_shares(
        &self,
        owner: &str,
        addressbook_id: &str,
    ) -> Result<Vec<AddressbookShare>, Error>;
    /// The shares in the home of `sharee` whose addressbook is not in the trash bin
    async fn addressbooks_shared_with(&self, sharee: &str) -> Result<Vec<AddressbookShare>, Error>;
    /// The share at `(sharee, name)` if its addressbook is not in the trash bin
    async fn get_addressbook_share(
        &self,
        sharee: &str,
        name: &str,
    ) -> Result<Option<AddressbookShare>, Error>;
    /// Only `displayname` and `color` apply to addressbooks
    async fn update_addressbook_share_view(
        &self,
        sharee: &str,
        name: &str,
        update: ShareViewUpdate,
    ) -> Result<(), Error>;

    /// Sets or removes the e-mail address of a principal. An address another principal had
    /// moves to this one (the newest claim of the identity provider wins).
    async fn set_principal_email(&self, principal: &str, email: Option<&str>) -> Result<(), Error>;
    async fn principal_email(&self, principal: &str) -> Result<Option<String>, Error>;
    /// The principal with exactly this e-mail address (compared in lowercase)
    async fn resolve_email(&self, email: &str) -> Result<Option<String>, Error>;
}
