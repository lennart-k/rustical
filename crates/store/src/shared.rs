//! The one place that decides what objects of a [`Resolved`] collection look like.
//!
//! It also decides how they are written. For [`ResolvedKind::Own`] everything passes through
//! to the stores unchanged;
//! for [`ResolvedKind::Copy`] reads return only the sharee's views (hidden objects left out) and
//! writes go through the conditional primitives of [`ShareStore`].
#![allow(clippy::missing_errors_doc)]
use crate::access::{Need, Resolved, ResolvedKind};
use crate::calendar_store::CalendarQuery;
use crate::{AddressbookStore, CalendarStore, Error, Preconditions, ShareStore};
pub use headers::{ETag, IfMatch, IfNoneMatch};
use rustical_ical::visibility::{self, FULL};
pub use rustical_ical::{AddressObject, CalendarObject};

pub use rustical_ical::visibility::{
    BUSY, FULL as FULL_LEVEL, HIDDEN, Policy, TITLE, TITLE_LOCATION, level_of, view,
};

/// The level `object` has in `resolved`: [`FULL`] in an own collection
#[must_use]
pub fn level(resolved: &Resolved, object: &CalendarObject) -> u8 {
    match resolved.kind() {
        ResolvedKind::Own => FULL,
        ResolvedKind::Copy { policy, .. } => level_of(object, *policy),
    }
}

/// `object` as `resolved` shows it; `None` if it is hidden there
#[must_use]
pub fn present(resolved: &Resolved, object: CalendarObject) -> Option<CalendarObject> {
    match resolved.kind() {
        ResolvedKind::Own => Some(object),
        ResolvedKind::Copy { policy, .. } => visibility::view_for(&object, *policy),
    }
}

fn present_all(
    resolved: &Resolved,
    objects: Vec<(String, CalendarObject)>,
) -> Vec<(String, CalendarObject)> {
    objects
        .into_iter()
        .filter_map(|(id, object)| Some((id, present(resolved, object)?)))
        .collect()
}

/// The live objects of a calendar
pub async fn objects(
    store: &dyn CalendarStore,
    resolved: &Resolved,
) -> Result<Vec<(String, CalendarObject)>, Error> {
    let objects = store.get_objects(resolved.owner(), resolved.id()).await?;
    Ok(present_all(resolved, objects))
}

/// One object of a calendar; through a share never one of the trash bin
pub async fn object(
    store: &dyn CalendarStore,
    resolved: &Resolved,
    object_id: &str,
    show_deleted: bool,
) -> Result<CalendarObject, Error> {
    let show_deleted = show_deleted && resolved.is_own();
    let object = store
        .get_object(resolved.owner(), resolved.id(), object_id, show_deleted)
        .await?;
    present(resolved, object).ok_or(Error::NotFound)
}

/// Changes since `synctoken`: objects that are hidden now count as removed
pub async fn sync_changes(
    store: &dyn CalendarStore,
    resolved: &Resolved,
    synctoken: i64,
) -> Result<(Vec<(String, CalendarObject)>, Vec<String>, i64), Error> {
    let (changed, mut removed, synctoken) = store
        .sync_changes(resolved.owner(), resolved.id(), synctoken)
        .await?;
    let mut visible = Vec::with_capacity(changed.len());
    for (id, object) in changed {
        match present(resolved, object) {
            Some(object) => visible.push((id, object)),
            None => removed.push(id),
        }
    }
    Ok((visible, removed, synctoken))
}

/// The prefiltered objects of a calendar-query. Filters must run on what this returns (the
/// views), never on the stored objects, or a filter would tell hidden content.
pub async fn query(
    store: &dyn CalendarStore,
    resolved: &Resolved,
    query: CalendarQuery,
) -> Result<Vec<(String, CalendarObject)>, Error> {
    let objects = store
        .calendar_query(resolved.owner(), resolved.id(), query)
        .await?;
    Ok(present_all(resolved, objects))
}

/// Writes an object.
///
/// In an own calendar exactly as [`CalendarStore::put_object`] (the caller checks
/// preconditions as before); through a share with [`ShareStore::put_copy_object`]. Returns the
/// `ETag` of what the writer sees now, if that is what was sent.
pub async fn put_object(
    store: &dyn CalendarStore,
    shares: &dyn ShareStore,
    resolved: &Resolved,
    object_id: &str,
    object: CalendarObject,
    overwrite: bool,
    preconditions: Preconditions<'_>,
) -> Result<Option<String>, Error> {
    resolved.check(Need::Write)?;
    match resolved.kind() {
        ResolvedKind::Own => {
            let etag = object.get_etag();
            store
                .put_object(
                    resolved.owner(),
                    resolved.id(),
                    object_id,
                    object,
                    overwrite,
                )
                .await?;
            Ok(Some(etag))
        }
        ResolvedKind::Copy { .. } => {
            shares
                .put_copy_object(
                    resolved.owner(),
                    resolved.id(),
                    object_id,
                    object,
                    preconditions,
                )
                .await
        }
    }
}

/// Writes several objects; through a share one after another with [`put_object`]'s rules
pub async fn put_objects(
    store: &dyn CalendarStore,
    shares: &dyn ShareStore,
    resolved: &Resolved,
    objects: Vec<(String, CalendarObject)>,
    overwrite: bool,
) -> Result<(), Error> {
    resolved.check(Need::Write)?;
    match resolved.kind() {
        ResolvedKind::Own => {
            store
                .put_objects(resolved.owner(), resolved.id(), objects, overwrite)
                .await
        }
        ResolvedKind::Copy { .. } => {
            for (object_id, object) in objects {
                shares
                    .put_copy_object(
                        resolved.owner(),
                        resolved.id(),
                        &object_id,
                        object,
                        Preconditions::default(),
                    )
                    .await?;
            }
            Ok(())
        }
    }
}

/// Deletes an object; through a share always into the owner's trash bin
pub async fn delete_object(
    store: &dyn CalendarStore,
    shares: &dyn ShareStore,
    resolved: &Resolved,
    object_id: &str,
    use_trashbin: bool,
    preconditions: Preconditions<'_>,
) -> Result<(), Error> {
    resolved.check(Need::Write)?;
    match resolved.kind() {
        ResolvedKind::Own => {
            store
                .delete_object(resolved.owner(), resolved.id(), object_id, use_trashbin)
                .await
        }
        ResolvedKind::Copy { .. } => {
            shares
                .delete_copy_object(resolved.owner(), resolved.id(), object_id, preconditions)
                .await
        }
    }
}

/// The live objects of an addressbook (contacts are always fully visible)
pub async fn address_objects(
    store: &dyn AddressbookStore,
    resolved: &Resolved,
) -> Result<Vec<(String, AddressObject)>, Error> {
    store.get_objects(resolved.owner(), resolved.id()).await
}

/// One object of an addressbook; through a share never one of the trash bin
pub async fn address_object(
    store: &dyn AddressbookStore,
    resolved: &Resolved,
    object_id: &str,
    show_deleted: bool,
) -> Result<AddressObject, Error> {
    store
        .get_object(
            resolved.owner(),
            resolved.id(),
            object_id,
            show_deleted && resolved.is_own(),
        )
        .await
}

/// Changes of an addressbook since `synctoken`
pub async fn address_sync_changes(
    store: &dyn AddressbookStore,
    resolved: &Resolved,
    synctoken: i64,
) -> Result<(Vec<(String, AddressObject)>, Vec<String>, i64), Error> {
    store
        .sync_changes(resolved.owner(), resolved.id(), synctoken)
        .await
}

/// Writes a contact; see [`put_object`]
pub async fn put_address_object(
    store: &dyn AddressbookStore,
    shares: &dyn ShareStore,
    resolved: &Resolved,
    object_id: &str,
    object: AddressObject,
    overwrite: bool,
    preconditions: Preconditions<'_>,
) -> Result<Option<String>, Error> {
    resolved.check(Need::Write)?;
    match resolved.kind() {
        ResolvedKind::Own => {
            let etag = object.get_etag();
            store
                .put_object(
                    resolved.owner(),
                    resolved.id(),
                    object_id,
                    object,
                    overwrite,
                )
                .await?;
            Ok(Some(etag))
        }
        ResolvedKind::Copy { .. } => {
            shares
                .put_copy_address_object(
                    resolved.owner(),
                    resolved.id(),
                    object_id,
                    object,
                    preconditions,
                )
                .await
        }
    }
}

/// Deletes a contact; through a share always into the owner's trash bin
pub async fn delete_address_object(
    store: &dyn AddressbookStore,
    shares: &dyn ShareStore,
    resolved: &Resolved,
    object_id: &str,
    use_trashbin: bool,
    preconditions: Preconditions<'_>,
) -> Result<(), Error> {
    resolved.check(Need::Write)?;
    match resolved.kind() {
        ResolvedKind::Own => {
            store
                .delete_object(resolved.owner(), resolved.id(), object_id, use_trashbin)
                .await
        }
        ResolvedKind::Copy { .. } => {
            shares
                .delete_copy_address_object(
                    resolved.owner(),
                    resolved.id(),
                    object_id,
                    preconditions,
                )
                .await
        }
    }
}
