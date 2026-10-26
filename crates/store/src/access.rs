//! Access decisions for collections (calendars and addressbooks).
//!
//! Every route that touches a collection asks two questions:
//!
//! 1. May the user use the home in the path at all? ([`check_home`], which is
//!    `Principal::is_principal`: the user themself or a group they are a member of.)
//! 2. What is the collection behind `(home, name)` and what does the route need from it?
//!    ([`resolve_calendar`] / [`resolve_addressbook`], then [`Resolved::check`].)
//!
//! Generic DAV routes do not know the user, only the path, so the path alone decides what
//! a collection is. Keeping this decision in one place lets later kinds of collections
//! (for example collections shared by another principal) plug in without touching every
//! route again.
use crate::{Addressbook, AddressbookStore, Calendar, CalendarStore, Error, auth::Principal};

/// What a route needs from a collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Need {
    /// Read the collection and its objects.
    Read,
    /// Create, change or delete objects.
    Write,
    /// Manage the collection itself (properties, deletion, restore).
    Own,
}

/// The kind of collection a path points to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedKind {
    /// A collection of the home itself (for a group home: the group's own collection).
    Own,
}

/// The collection behind a path `(home, name)`.
///
/// Only this module creates values of this type, so holding one means the path was resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    owner: String,
    id: String,
    kind: ResolvedKind,
}

impl Resolved {
    /// The principal the collection belongs to.
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// The id of the collection in the owner's namespace.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub const fn kind(&self) -> &ResolvedKind {
        &self.kind
    }

    /// Whether the collection grants what a route needs.
    #[must_use]
    pub const fn allows(&self, need: Need) -> bool {
        match self.kind {
            // The home's own collection: everything, as before sharing existed
            ResolvedKind::Own => matches!(need, Need::Read | Need::Write | Need::Own),
        }
    }

    /// [`Self::allows`] as a result.
    ///
    /// # Errors
    /// [`Error::Forbidden`] if the route may not proceed.
    pub const fn check(&self, need: Need) -> Result<(), Error> {
        if self.allows(need) {
            Ok(())
        } else {
            Err(Error::Forbidden)
        }
    }

    const fn own(owner: String, id: String) -> Self {
        Self {
            owner,
            id,
            kind: ResolvedKind::Own,
        }
    }
}

/// Whether `user` may use paths below `home` at all.
#[must_use]
pub fn may_use_home(user: &Principal, home: &str) -> bool {
    user.is_principal(home)
}

/// [`may_use_home`] as a result.
///
/// # Errors
/// [`Error::Forbidden`] for an authenticated user without access to `home`.
pub fn check_home(user: &Principal, home: &str) -> Result<(), Error> {
    if may_use_home(user, home) {
        Ok(())
    } else {
        Err(Error::Forbidden)
    }
}

/// Resolves the calendar at `(home, name)`.
///
/// Returns the calendar as the path presents it together with what it is.
///
/// # Errors
/// [`Error::NotFound`] if there is no such calendar, or any error of the store.
pub async fn resolve_calendar(
    store: &dyn CalendarStore,
    home: &str,
    name: &str,
    show_deleted: bool,
) -> Result<(Resolved, Calendar), Error> {
    let calendar = store.get_calendar(home, name, show_deleted).await?;
    Ok((
        Resolved::own(calendar.principal.clone(), calendar.id.clone()),
        calendar,
    ))
}

/// Resolves the addressbook at `(home, name)`.
///
/// Returns the addressbook as the path presents it together with what it is.
///
/// # Errors
/// [`Error::NotFound`] if there is no such addressbook, or any error of the store.
pub async fn resolve_addressbook(
    store: &dyn AddressbookStore,
    home: &str,
    name: &str,
    show_deleted: bool,
) -> Result<(Resolved, Addressbook), Error> {
    let addressbook = store.get_addressbook(home, name, show_deleted).await?;
    Ok((
        Resolved::own(addressbook.principal.clone(), addressbook.id.clone()),
        addressbook,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::PrincipalType;

    fn user(id: &str, memberships: &[&str]) -> Principal {
        Principal {
            id: id.to_owned(),
            displayname: None,
            principal_type: PrincipalType::Individual,
            password: None,
            memberships: memberships.iter().map(|&m| m.to_owned()).collect(),
        }
    }

    #[test]
    fn home_access() {
        let alice = user("alice", &["group.team"]);
        assert!(check_home(&alice, "alice").is_ok());
        assert!(check_home(&alice, "group.team").is_ok());
        assert!(matches!(check_home(&alice, "bob"), Err(Error::Forbidden)));
    }

    #[test]
    fn own_allows_everything() {
        let resolved = Resolved::own("alice".to_owned(), "cal".to_owned());
        assert_eq!(resolved.owner(), "alice");
        assert_eq!(resolved.id(), "cal");
        for need in [Need::Read, Need::Write, Need::Own] {
            assert!(resolved.check(need).is_ok());
        }
    }
}
