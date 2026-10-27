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
use crate::{
    Access, Addressbook, AddressbookShare, AddressbookStore, Calendar, CalendarShare,
    CalendarStore, Error, ShareStore, auth::Principal,
};
use rustical_ical::visibility::{FULL, Policy};

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
    /// A collection another principal shares with the home. Everyone using it sees it as a
    /// sharee: objects as `policy` (the owner's) shows them, writes only with `Access::Write`.
    Copy { access: Access, policy: Policy },
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
            ResolvedKind::Copy { access, .. } => match need {
                Need::Read => true,
                Need::Write => matches!(access, Access::Write),
                Need::Own => false,
            },
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

    #[must_use]
    pub const fn is_own(&self) -> bool {
        matches!(self.kind, ResolvedKind::Own)
    }

    const fn own(owner: String, id: String) -> Self {
        Self {
            owner,
            id,
            kind: ResolvedKind::Own,
        }
    }

    const fn copy(owner: String, id: String, access: Access, policy: Policy) -> Self {
        Self {
            owner,
            id,
            kind: ResolvedKind::Copy { access, policy },
        }
    }
}

#[cfg(any(test, feature = "test"))]
impl Resolved {
    /// A resolved collection for tests that cannot resolve one through a store
    #[must_use]
    pub const fn for_tests(owner: String, id: String, kind: ResolvedKind) -> Self {
        Self { owner, id, kind }
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

/// Resolves the calendar at `(home, name)`: a calendar of the home, or a calendar shared
/// with the home.
///
/// Returns the calendar as the path presents it (for a share with the sharee's display name,
/// color and order) together with what it is. A shared calendar in the owner's trash bin does
/// not exist for the sharee, whatever `show_deleted` says.
///
/// # Errors
/// [`Error::NotFound`] if there is no such calendar, or any error of the stores.
pub async fn resolve_calendar(
    shares: &dyn ShareStore,
    store: &dyn CalendarStore,
    home: &str,
    name: &str,
    show_deleted: bool,
) -> Result<(Resolved, Calendar), Error> {
    match store.get_calendar(home, name, show_deleted).await {
        Ok(calendar) => {
            return Ok((
                Resolved::own(calendar.principal.clone(), calendar.id.clone()),
                calendar,
            ));
        }
        Err(Error::NotFound) => {}
        Err(err) => return Err(err),
    }
    let share = shares
        .get_calendar_share(home, name)
        .await?
        .ok_or(Error::NotFound)?;
    shared_calendar(shares, store, share).await
}

/// A share in the home of its sharee as [`resolve_calendar`] presents it
async fn shared_calendar(
    shares: &dyn ShareStore,
    store: &dyn CalendarStore,
    share: CalendarShare,
) -> Result<(Resolved, Calendar), Error> {
    let mut calendar = store
        .get_calendar(&share.owner, &share.cal_id, false)
        .await?;
    let policy = shares.get_visibility(&share.owner).await?;
    if share.displayname.is_some() {
        calendar.meta.displayname = share.displayname;
    }
    if share.color.is_some() {
        calendar.meta.color = share.color;
    }
    if let Some(order) = share.order {
        calendar.meta.order = order;
    }
    Ok((
        Resolved::copy(share.owner, share.cal_id, share.access, policy),
        calendar,
    ))
}

/// The live calendars of `home` itself, resolved.
///
/// # Errors
/// Any error of the store.
pub async fn own_calendars(
    store: &dyn CalendarStore,
    home: &str,
) -> Result<Vec<(Resolved, Calendar)>, Error> {
    Ok(store
        .get_calendars(home)
        .await?
        .into_iter()
        .map(|calendar| {
            (
                Resolved::own(calendar.principal.clone(), calendar.id.clone()),
                calendar,
            )
        })
        .collect())
}

/// The live addressbooks of `home` itself, resolved.
///
/// # Errors
/// Any error of the store.
pub async fn own_addressbooks(
    store: &dyn AddressbookStore,
    home: &str,
) -> Result<Vec<(Resolved, Addressbook)>, Error> {
    Ok(store
        .get_addressbooks(home)
        .await?
        .into_iter()
        .map(|addressbook| {
            (
                Resolved::own(addressbook.principal.clone(), addressbook.id.clone()),
                addressbook,
            )
        })
        .collect())
}

/// The principals the live calendars of `owner` are shared with (users and groups).
///
/// They may see the principal `owner` itself: its name, not its homes or memberships.
///
/// # Errors
/// Any error of the stores.
pub async fn calendar_sharees(
    shares: &dyn ShareStore,
    store: &dyn CalendarStore,
    owner: &str,
) -> Result<Vec<String>, Error> {
    let mut principals = vec![];
    for calendar in store.get_calendars(owner).await? {
        for share in shares.list_calendar_shares(owner, &calendar.id).await? {
            if !principals.contains(&share.sharee) {
                principals.push(share.sharee);
            }
        }
    }
    Ok(principals)
}

/// The principals the live addressbooks of `owner` are shared with; see [`calendar_sharees`].
///
/// # Errors
/// Any error of the stores.
pub async fn addressbook_sharees(
    shares: &dyn ShareStore,
    store: &dyn AddressbookStore,
    owner: &str,
) -> Result<Vec<String>, Error> {
    let mut principals = vec![];
    for addressbook in store.get_addressbooks(owner).await? {
        for share in shares
            .list_addressbook_shares(owner, &addressbook.id)
            .await?
        {
            if !principals.contains(&share.sharee) {
                principals.push(share.sharee);
            }
        }
    }
    Ok(principals)
}

/// Whether `user` may see the principal resource of `owner`.
///
/// It may see its own (or one of its groups'), and that of someone who shares with it or one of
/// its groups (`sharees` of `owner`). A sharee sees only who the owner is (see the principal
/// resources).
#[must_use]
pub fn may_see_principal(user: &Principal, owner: &str, sharees: &[String]) -> bool {
    may_use_home(user, owner) || sharees.iter().any(|sharee| user.is_principal(sharee))
}

/// The calendars shared with `home`, as [`resolve_calendar`] presents them, with their names in
/// the home. A share whose calendar cannot be read is left out (never an error for the listing).
///
/// # Errors
/// Any error of the share store listing the shares.
pub async fn shared_calendars(
    shares: &dyn ShareStore,
    store: &dyn CalendarStore,
    home: &str,
) -> Result<Vec<(String, Resolved, Calendar)>, Error> {
    let mut result = vec![];
    for share in shares.calendars_shared_with(home).await? {
        let name = share.name.clone();
        match shared_calendar(shares, store, share).await {
            Ok((resolved, calendar)) => result.push((name, resolved, calendar)),
            Err(err) => tracing::warn!("Skipping shared calendar {home}/{name}: {err}"),
        }
    }
    Ok(result)
}

/// Resolves the addressbook at `(home, name)`: an addressbook of the home, or one shared with
/// the home. See [`resolve_calendar`].
///
/// # Errors
/// [`Error::NotFound`] if there is no such addressbook, or any error of the stores.
pub async fn resolve_addressbook(
    shares: &dyn ShareStore,
    store: &dyn AddressbookStore,
    home: &str,
    name: &str,
    show_deleted: bool,
) -> Result<(Resolved, Addressbook), Error> {
    match store.get_addressbook(home, name, show_deleted).await {
        Ok(addressbook) => {
            return Ok((
                Resolved::own(addressbook.principal.clone(), addressbook.id.clone()),
                addressbook,
            ));
        }
        Err(Error::NotFound) => {}
        Err(err) => return Err(err),
    }
    let share = shares
        .get_addressbook_share(home, name)
        .await?
        .ok_or(Error::NotFound)?;
    shared_addressbook(store, share).await
}

async fn shared_addressbook(
    store: &dyn AddressbookStore,
    share: AddressbookShare,
) -> Result<(Resolved, Addressbook), Error> {
    let mut addressbook = store
        .get_addressbook(&share.owner, &share.addressbook_id, false)
        .await?;
    if share.displayname.is_some() {
        addressbook.displayname = share.displayname;
    }
    Ok((
        // Contacts have no CLASS: always fully visible
        Resolved::copy(
            share.owner,
            share.addressbook_id,
            share.access,
            Policy::new(FULL, FULL).expect("valid levels"),
        ),
        addressbook,
    ))
}

/// The addressbooks shared with `home`; see [`shared_calendars`].
///
/// # Errors
/// Any error of the share store listing the shares.
pub async fn shared_addressbooks(
    shares: &dyn ShareStore,
    store: &dyn AddressbookStore,
    home: &str,
) -> Result<Vec<(String, Resolved, Addressbook)>, Error> {
    let mut result = vec![];
    for share in shares.addressbooks_shared_with(home).await? {
        let name = share.name.clone();
        match shared_addressbook(store, share).await {
            Ok((resolved, addressbook)) => result.push((name, resolved, addressbook)),
            Err(err) => tracing::warn!("Skipping shared addressbook {home}/{name}: {err}"),
        }
    }
    Ok(result)
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
    fn copy_allows_by_access() {
        let policy = Policy::default();
        let read = Resolved::for_tests(
            "alice".to_owned(),
            "cal".to_owned(),
            ResolvedKind::Copy {
                access: Access::Read,
                policy,
            },
        );
        assert!(read.allows(Need::Read));
        assert!(!read.allows(Need::Write));
        assert!(!read.allows(Need::Own));
        let write = Resolved::for_tests(
            "alice".to_owned(),
            "cal".to_owned(),
            ResolvedKind::Copy {
                access: Access::Write,
                policy,
            },
        );
        assert!(write.allows(Need::Read));
        assert!(write.allows(Need::Write));
        assert!(!write.allows(Need::Own));
        assert!(!write.is_own());
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
