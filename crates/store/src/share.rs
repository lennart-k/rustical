//! Sharing collections with other principals.
//!
//! A share gives one principal (the sharee, a user or a group) access to a calendar or an
//! addressbook of another principal (the owner). The shared collection appears in the
//! sharee's home under its own name, with the sharee's own display name and color.
use crate::Error;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use headers::{ETag, IfMatch, IfNoneMatch};
use rustical_ical::visibility::Policy;
use rustical_ical::{AddressObject, CalendarObject};
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

/// Why the store refused a write through a share with write access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CopyRule {
    #[error("an object of the owner's trash bin has this name")]
    TrashHref,
    #[error("another object of the calendar has this UID")]
    ForeignUid,
    #[error("the UID of an existing object cannot change")]
    UidChanged,
    #[error("only fully visible objects can be changed")]
    Level,
}

/// The conditions of a write through a share, compared with the object as the sharee sees it.
#[derive(Debug, Clone, Copy, Default)]
pub struct Preconditions<'a> {
    pub if_match: Option<&'a IfMatch>,
    pub if_none_match: Option<&'a IfNoneMatch>,
}

impl Preconditions<'_> {
    /// Checks the conditions for the visible object with `etag` (`None`: there is none).
    ///
    /// # Errors
    /// [`Error::PreconditionFailed`] unless the conditions hold.
    pub fn check(&self, etag: Option<&str>) -> Result<(), Error> {
        let etag: Option<ETag> = etag.and_then(|etag| etag.parse().ok());
        if let Some(if_match) = self.if_match
            && etag
                .as_ref()
                .is_none_or(|etag| !if_match.precondition_passes(etag))
        {
            return Err(Error::PreconditionFailed);
        }
        if let Some(if_none_match) = self.if_none_match
            && etag
                .as_ref()
                .is_some_and(|etag| !if_none_match.precondition_passes(etag))
        {
            return Err(Error::PreconditionFailed);
        }
        Ok(())
    }
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
pub trait ShareStore: std::fmt::Debug + Send + Sync + 'static {
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

    /// How objects of `principal` with CLASS:PRIVATE/CONFIDENTIAL appear in its shares
    async fn get_visibility(&self, principal: &str) -> Result<Policy, Error>;
    /// Changes the policy; every object of a shared calendar whose view depends on it is
    /// logged as changed, so sharees pick the new views up with their next sync.
    async fn set_visibility(&self, principal: &str, policy: Policy) -> Result<(), Error>;

    /// Writes an object of `(owner, cal_id)` for a sharee with write access, in one transaction:
    /// only a fully visible, live object can be replaced (its UID unchanged, its alarms kept);
    /// a new object needs a name and a UID unused in the calendar, also in the trash bin.
    /// The conditions are checked against the sharee's view. Returns the `ETag` of that view
    /// if what is stored is what was sent.
    async fn put_copy_object(
        &self,
        owner: &str,
        cal_id: &str,
        object_id: &str,
        object: CalendarObject,
        preconditions: Preconditions<'_>,
    ) -> Result<Option<String>, Error>;
    /// Moves a fully visible object of `(owner, cal_id)` to the trash bin for a sharee
    async fn delete_copy_object(
        &self,
        owner: &str,
        cal_id: &str,
        object_id: &str,
        preconditions: Preconditions<'_>,
    ) -> Result<(), Error>;
    /// [`Self::put_copy_object`] for addressbooks (contacts are always fully visible)
    async fn put_copy_address_object(
        &self,
        owner: &str,
        addressbook_id: &str,
        object_id: &str,
        object: AddressObject,
        preconditions: Preconditions<'_>,
    ) -> Result<Option<String>, Error>;
    /// [`Self::delete_copy_object`] for addressbooks
    async fn delete_copy_address_object(
        &self,
        owner: &str,
        addressbook_id: &str,
        object_id: &str,
        preconditions: Preconditions<'_>,
    ) -> Result<(), Error>;
}
