use crate::calendar_store::ObjectColumns;
use crate::{BEGIN_IMMEDIATE, ChangeOperation, SqliteAddressbookStore, SqliteCalendarStore};
use async_trait::async_trait;
use derive_more::Constructor;
use rustical_ical::visibility::{self, Classes, FULL, HIDDEN, Policy};
use rustical_ical::{AddressObject, CalendarObject};
use rustical_store::{
    Access, AddressbookShare, CalendarShare, CollectionOperation, CollectionOperationInfo,
    CopyRule, Error, Preconditions, ShareRejection, ShareStore, ShareViewUpdate,
};
use sqlx::types::chrono::NaiveDateTime;
use sqlx::{Sqlite, SqliteConnection, SqlitePool, Transaction};
use tokio::sync::mpsc::Sender;
use tracing::{error_span, instrument, warn};

#[cfg(test)]
mod copy_tests;
#[cfg(test)]
mod tests;

/// Calendar ids with this prefix belong to the birthday calendars of addressbooks
const BIRTHDAYS_PREFIX: &str = "_birthdays_";

#[derive(Debug, Clone, Constructor)]
pub struct SqliteShareStore {
    db: SqlitePool,
    // Push notifications for changes made through shares
    sender: Sender<CollectionOperation>,
}

struct CalendarShareRow {
    owner: String,
    cal_id: String,
    sharee: String,
    name: String,
    access: String,
    displayname: Option<String>,
    color: Option<String>,
    order: Option<i64>,
    created_at: NaiveDateTime,
    created_by: Option<String>,
}

impl TryFrom<CalendarShareRow> for CalendarShare {
    type Error = Error;

    fn try_from(row: CalendarShareRow) -> Result<Self, Error> {
        Ok(Self {
            owner: row.owner,
            cal_id: row.cal_id,
            sharee: row.sharee,
            name: row.name,
            access: row.access.parse()?,
            displayname: row.displayname,
            color: row.color,
            order: row.order,
            created_at: row.created_at,
            created_by: row.created_by,
        })
    }
}

struct AddressbookShareRow {
    owner: String,
    addressbook_id: String,
    sharee: String,
    name: String,
    access: String,
    displayname: Option<String>,
    color: Option<String>,
    created_at: NaiveDateTime,
    created_by: Option<String>,
}

impl TryFrom<AddressbookShareRow> for AddressbookShare {
    type Error = Error;

    fn try_from(row: AddressbookShareRow) -> Result<Self, Error> {
        Ok(Self {
            owner: row.owner,
            addressbook_id: row.addressbook_id,
            sharee: row.sharee,
            name: row.name,
            access: row.access.parse()?,
            displayname: row.displayname,
            color: row.color,
            created_at: row.created_at,
            created_by: row.created_by,
        })
    }
}

fn collect<R, T: TryFrom<R, Error = Error>>(rows: Vec<R>) -> Result<Vec<T>, Error> {
    rows.into_iter().map(T::try_from).collect()
}

/// Whether a collection name in the home of `sharee` is already used by a share.
/// Called by the calendar and addressbook stores before they create a collection.
pub async fn calendar_name_shared(
    tx: &mut Transaction<'_, Sqlite>,
    sharee: &str,
    name: &str,
) -> Result<bool, Error> {
    Ok(sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM calendar_shares WHERE (sharee, name) = (?, ?)) AS "taken!: bool""#,
        sharee,
        name
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(crate::Error::from)?)
}

/// See [`calendar_name_shared`]
pub async fn addressbook_name_shared(
    tx: &mut Transaction<'_, Sqlite>,
    sharee: &str,
    name: &str,
) -> Result<bool, Error> {
    Ok(sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM addressbook_shares WHERE (sharee, name) = (?, ?)) AS "taken!: bool""#,
        sharee,
        name
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(crate::Error::from)?)
}

/// Sets or removes the e-mail address of a principal; an address another principal had moves
/// to this one (the newest claim of the identity provider wins). The one implementation for the
/// share store and the principal store.
pub async fn set_principal_email(
    db: &SqlitePool,
    principal: &str,
    email: Option<&str>,
) -> Result<(), Error> {
    let mut tx = db
        .begin_with(BEGIN_IMMEDIATE)
        .await
        .map_err(crate::Error::from)?;
    match email.map(str::trim).filter(|email| !email.is_empty()) {
        None => {
            sqlx::query!(
                r#"DELETE FROM principal_emails WHERE principal = ?"#,
                principal
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }
        Some(email) => {
            let email = email.to_lowercase();
            // The newest claim wins: the identity provider is the authority on addresses
            let moved = sqlx::query!(
                r#"DELETE FROM principal_emails WHERE email = ? AND principal != ?"#,
                email,
                principal
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
            if moved.rows_affected() > 0 {
                warn!("E-mail address {email} moved to principal {principal}");
            }
            sqlx::query!(
                r#"INSERT INTO principal_emails (principal, email) VALUES (?, ?)
                    ON CONFLICT(principal) DO UPDATE SET email = excluded.email"#,
                principal,
                email
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }
    }
    tx.commit().await.map_err(crate::Error::from)?;
    Ok(())
}

impl SqliteShareStore {
    /// Resolves a principal id or an e-mail address to a principal id.
    async fn resolve_sharee(
        tx: &mut Transaction<'_, Sqlite>,
        input: &str,
    ) -> Result<String, Error> {
        let input = input.trim();
        let by_id = sqlx::query_scalar!(r#"SELECT id FROM principals WHERE id = ?"#, input)
            .fetch_optional(&mut **tx)
            .await
            .map_err(crate::Error::from)?;
        let email = input.to_lowercase();
        let by_email = sqlx::query_scalar!(
            r#"SELECT principal FROM principal_emails WHERE email = ?"#,
            email
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(crate::Error::from)?;
        match (by_id, by_email) {
            (Some(id), Some(other)) if id != other => Err(ShareRejection::AmbiguousSharee.into()),
            (Some(id), _) | (None, Some(id)) => Ok(id),
            (None, None) => Err(ShareRejection::UnknownSharee.into()),
        }
    }

    /// The rules every share obeys, whatever is shared
    async fn check_sharee(
        tx: &mut Transaction<'_, Sqlite>,
        owner: &str,
        sharee: &str,
    ) -> Result<(), Error> {
        if owner == sharee {
            return Err(ShareRejection::Owner.into());
        }
        let member = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM memberships WHERE (principal, member_of) = (?, ?)) AS "member!: bool""#,
            sharee,
            owner
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(crate::Error::from)?;
        if member {
            return Err(ShareRejection::OwnerMember.into());
        }
        Ok(())
    }

    async fn free_calendar_name(
        tx: &mut Transaction<'_, Sqlite>,
        sharee: &str,
        base: &str,
    ) -> Result<String, Error> {
        let mut candidate = base.to_owned();
        for n in 2.. {
            let taken = sqlx::query_scalar!(
                r#"SELECT (
                    EXISTS(SELECT 1 FROM calendars WHERE (principal, id) = (?1, ?2))
                    OR EXISTS(SELECT 1 FROM calendar_shares WHERE (sharee, name) = (?1, ?2))
                ) AS "taken!: bool""#,
                sharee,
                candidate
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(crate::Error::from)?;
            if !taken {
                break;
            }
            candidate = format!("{base}-{n}");
        }
        Ok(candidate)
    }

    async fn free_addressbook_name(
        tx: &mut Transaction<'_, Sqlite>,
        sharee: &str,
        base: &str,
    ) -> Result<String, Error> {
        let mut candidate = base.to_owned();
        for n in 2.. {
            let taken = sqlx::query_scalar!(
                r#"SELECT (
                    EXISTS(SELECT 1 FROM addressbooks WHERE (principal, id) = (?1, ?2))
                    OR EXISTS(SELECT 1 FROM addressbook_shares WHERE (sharee, name) = (?1, ?2))
                ) AS "taken!: bool""#,
                sharee,
                candidate
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(crate::Error::from)?;
            if !taken {
                break;
            }
            candidate = format!("{base}-{n}");
        }
        Ok(candidate)
    }

    async fn _get_calendar_share(
        tx: &mut Transaction<'_, Sqlite>,
        owner: &str,
        cal_id: &str,
        sharee: &str,
    ) -> Result<Option<CalendarShare>, Error> {
        sqlx::query_as!(
            CalendarShareRow,
            r#"SELECT owner, cal_id, sharee, name, access, displayname, color, "order", created_at AS "created_at: NaiveDateTime", created_by
                FROM calendar_shares WHERE (owner, cal_id, sharee) = (?, ?, ?)"#,
            owner,
            cal_id,
            sharee
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(crate::Error::from)?
        .map(CalendarShare::try_from)
        .transpose()
    }

    async fn _get_addressbook_share(
        tx: &mut Transaction<'_, Sqlite>,
        owner: &str,
        addressbook_id: &str,
        sharee: &str,
    ) -> Result<Option<AddressbookShare>, Error> {
        sqlx::query_as!(
            AddressbookShareRow,
            r#"SELECT owner, addressbook_id, sharee, name, access, displayname, color, created_at AS "created_at: NaiveDateTime", created_by
                FROM addressbook_shares WHERE (owner, addressbook_id, sharee) = (?, ?, ?)"#,
            owner,
            addressbook_id,
            sharee
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(crate::Error::from)?
        .map(AddressbookShare::try_from)
        .transpose()
    }
}

#[async_trait]
impl ShareStore for SqliteShareStore {
    #[instrument]
    async fn share_calendar(
        &self,
        owner: &str,
        cal_id: &str,
        sharee: &str,
        access: Access,
        created_by: Option<&str>,
    ) -> Result<CalendarShare, Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        if cal_id.starts_with(BIRTHDAYS_PREFIX) {
            return Err(ShareRejection::NotShareable.into());
        }
        let subscription = sqlx::query_scalar!(
            r#"SELECT subscription_url IS NOT NULL AS "subscription!: bool"
                FROM calendars WHERE (principal, id) = (?, ?) AND deleted_at IS NULL"#,
            owner,
            cal_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::Error::from)?
        .ok_or(Error::NotFound)?;
        if subscription {
            return Err(ShareRejection::NotShareable.into());
        }
        let sharee = Self::resolve_sharee(&mut tx, sharee).await?;
        Self::check_sharee(&mut tx, owner, &sharee).await?;

        let access_str = access.as_str();
        if Self::_get_calendar_share(&mut tx, owner, cal_id, &sharee)
            .await?
            .is_some()
        {
            sqlx::query!(
                r#"UPDATE calendar_shares SET access = ? WHERE (owner, cal_id, sharee) = (?, ?, ?)"#,
                access_str,
                owner,
                cal_id,
                sharee
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        } else {
            let name =
                Self::free_calendar_name(&mut tx, &sharee, &format!("{cal_id}_shared_by_{owner}"))
                    .await?;
            sqlx::query!(
                r#"INSERT INTO calendar_shares (owner, cal_id, sharee, name, access, created_by)
                    VALUES (?, ?, ?, ?, ?, ?)"#,
                owner,
                cal_id,
                sharee,
                name,
                access_str,
                created_by
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }
        let share = Self::_get_calendar_share(&mut tx, owner, cal_id, &sharee)
            .await?
            .ok_or(Error::NotFound)?;
        tx.commit().await.map_err(crate::Error::from)?;
        Ok(share)
    }

    #[instrument]
    async fn unshare_calendar(&self, owner: &str, cal_id: &str, sharee: &str) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"DELETE FROM calendar_shares WHERE (owner, cal_id, sharee) = (?, ?, ?)"#,
            owner,
            cal_id,
            sharee
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    #[instrument]
    async fn set_calendar_share_access(
        &self,
        owner: &str,
        cal_id: &str,
        sharee: &str,
        access: Access,
    ) -> Result<(), Error> {
        let access = access.as_str();
        let result = sqlx::query!(
            r#"UPDATE calendar_shares SET access = ? WHERE (owner, cal_id, sharee) = (?, ?, ?)"#,
            access,
            owner,
            cal_id,
            sharee
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    #[instrument]
    async fn list_calendar_shares(
        &self,
        owner: &str,
        cal_id: &str,
    ) -> Result<Vec<CalendarShare>, Error> {
        collect(
            sqlx::query_as!(
                CalendarShareRow,
                r#"SELECT owner, cal_id, sharee, name, access, displayname, color, "order", created_at AS "created_at: NaiveDateTime", created_by
                    FROM calendar_shares WHERE (owner, cal_id) = (?, ?) ORDER BY sharee"#,
                owner,
                cal_id
            )
            .fetch_all(&self.db)
            .await
            .map_err(crate::Error::from)?,
        )
    }

    #[instrument]
    async fn calendars_shared_with(&self, sharee: &str) -> Result<Vec<CalendarShare>, Error> {
        collect(
            sqlx::query_as!(
                CalendarShareRow,
                r#"SELECT s.owner, s.cal_id, s.sharee, s.name, s.access, s.displayname, s.color, s."order", s.created_at AS "created_at: NaiveDateTime", s.created_by
                    FROM calendar_shares AS s
                    JOIN calendars AS c ON (c.principal, c.id) = (s.owner, s.cal_id)
                    WHERE s.sharee = ? AND c.deleted_at IS NULL
                    ORDER BY s.name"#,
                sharee
            )
            .fetch_all(&self.db)
            .await
            .map_err(crate::Error::from)?,
        )
    }

    #[instrument]
    async fn get_calendar_share(
        &self,
        sharee: &str,
        name: &str,
    ) -> Result<Option<CalendarShare>, Error> {
        sqlx::query_as!(
            CalendarShareRow,
            r#"SELECT s.owner, s.cal_id, s.sharee, s.name, s.access, s.displayname, s.color, s."order", s.created_at AS "created_at: NaiveDateTime", s.created_by
                FROM calendar_shares AS s
                JOIN calendars AS c ON (c.principal, c.id) = (s.owner, s.cal_id)
                WHERE (s.sharee, s.name) = (?, ?) AND c.deleted_at IS NULL"#,
            sharee,
            name
        )
        .fetch_optional(&self.db)
        .await
        .map_err(crate::Error::from)?
        .map(CalendarShare::try_from)
        .transpose()
    }

    #[instrument]
    async fn update_calendar_share_view(
        &self,
        sharee: &str,
        name: &str,
        update: ShareViewUpdate,
    ) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM calendar_shares WHERE (sharee, name) = (?, ?)) AS "exists!: bool""#,
            sharee,
            name
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::Error::from)?;
        if !exists {
            return Err(Error::NotFound);
        }
        // Only the changed columns are written
        if let Some(displayname) = update.displayname {
            sqlx::query!(
                r#"UPDATE calendar_shares SET displayname = ? WHERE (sharee, name) = (?, ?)"#,
                displayname,
                sharee,
                name
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }
        if let Some(color) = update.color {
            sqlx::query!(
                r#"UPDATE calendar_shares SET color = ? WHERE (sharee, name) = (?, ?)"#,
                color,
                sharee,
                name
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }
        if let Some(order) = update.order {
            sqlx::query!(
                r#"UPDATE calendar_shares SET "order" = ? WHERE (sharee, name) = (?, ?)"#,
                order,
                sharee,
                name
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }
        tx.commit().await.map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn share_addressbook(
        &self,
        owner: &str,
        addressbook_id: &str,
        sharee: &str,
        access: Access,
        created_by: Option<&str>,
    ) -> Result<AddressbookShare, Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM addressbooks WHERE (principal, id) = (?, ?) AND deleted_at IS NULL) AS "exists!: bool""#,
            owner,
            addressbook_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::Error::from)?;
        if !exists {
            return Err(Error::NotFound);
        }
        let sharee = Self::resolve_sharee(&mut tx, sharee).await?;
        Self::check_sharee(&mut tx, owner, &sharee).await?;

        let access_str = access.as_str();
        if Self::_get_addressbook_share(&mut tx, owner, addressbook_id, &sharee)
            .await?
            .is_some()
        {
            sqlx::query!(
                r#"UPDATE addressbook_shares SET access = ? WHERE (owner, addressbook_id, sharee) = (?, ?, ?)"#,
                access_str,
                owner,
                addressbook_id,
                sharee
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        } else {
            let name = Self::free_addressbook_name(
                &mut tx,
                &sharee,
                &format!("{addressbook_id}_shared_by_{owner}"),
            )
            .await?;
            sqlx::query!(
                r#"INSERT INTO addressbook_shares (owner, addressbook_id, sharee, name, access, created_by)
                    VALUES (?, ?, ?, ?, ?, ?)"#,
                owner,
                addressbook_id,
                sharee,
                name,
                access_str,
                created_by
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }
        let share = Self::_get_addressbook_share(&mut tx, owner, addressbook_id, &sharee)
            .await?
            .ok_or(Error::NotFound)?;
        tx.commit().await.map_err(crate::Error::from)?;
        Ok(share)
    }

    #[instrument]
    async fn unshare_addressbook(
        &self,
        owner: &str,
        addressbook_id: &str,
        sharee: &str,
    ) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"DELETE FROM addressbook_shares WHERE (owner, addressbook_id, sharee) = (?, ?, ?)"#,
            owner,
            addressbook_id,
            sharee
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    #[instrument]
    async fn set_addressbook_share_access(
        &self,
        owner: &str,
        addressbook_id: &str,
        sharee: &str,
        access: Access,
    ) -> Result<(), Error> {
        let access = access.as_str();
        let result = sqlx::query!(
            r#"UPDATE addressbook_shares SET access = ? WHERE (owner, addressbook_id, sharee) = (?, ?, ?)"#,
            access,
            owner,
            addressbook_id,
            sharee
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    #[instrument]
    async fn list_addressbook_shares(
        &self,
        owner: &str,
        addressbook_id: &str,
    ) -> Result<Vec<AddressbookShare>, Error> {
        collect(
            sqlx::query_as!(
                AddressbookShareRow,
                r#"SELECT owner, addressbook_id, sharee, name, access, displayname, color, created_at AS "created_at: NaiveDateTime", created_by
                    FROM addressbook_shares WHERE (owner, addressbook_id) = (?, ?) ORDER BY sharee"#,
                owner,
                addressbook_id
            )
            .fetch_all(&self.db)
            .await
            .map_err(crate::Error::from)?,
        )
    }

    #[instrument]
    async fn addressbooks_shared_with(&self, sharee: &str) -> Result<Vec<AddressbookShare>, Error> {
        collect(
            sqlx::query_as!(
                AddressbookShareRow,
                r#"SELECT s.owner, s.addressbook_id, s.sharee, s.name, s.access, s.displayname, s.color, s.created_at AS "created_at: NaiveDateTime", s.created_by
                    FROM addressbook_shares AS s
                    JOIN addressbooks AS a ON (a.principal, a.id) = (s.owner, s.addressbook_id)
                    WHERE s.sharee = ? AND a.deleted_at IS NULL
                    ORDER BY s.name"#,
                sharee
            )
            .fetch_all(&self.db)
            .await
            .map_err(crate::Error::from)?,
        )
    }

    #[instrument]
    async fn get_addressbook_share(
        &self,
        sharee: &str,
        name: &str,
    ) -> Result<Option<AddressbookShare>, Error> {
        sqlx::query_as!(
            AddressbookShareRow,
            r#"SELECT s.owner, s.addressbook_id, s.sharee, s.name, s.access, s.displayname, s.color, s.created_at AS "created_at: NaiveDateTime", s.created_by
                FROM addressbook_shares AS s
                JOIN addressbooks AS a ON (a.principal, a.id) = (s.owner, s.addressbook_id)
                WHERE (s.sharee, s.name) = (?, ?) AND a.deleted_at IS NULL"#,
            sharee,
            name
        )
        .fetch_optional(&self.db)
        .await
        .map_err(crate::Error::from)?
        .map(AddressbookShare::try_from)
        .transpose()
    }

    #[instrument]
    async fn update_addressbook_share_view(
        &self,
        sharee: &str,
        name: &str,
        update: ShareViewUpdate,
    ) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM addressbook_shares WHERE (sharee, name) = (?, ?)) AS "exists!: bool""#,
            sharee,
            name
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::Error::from)?;
        if !exists {
            return Err(Error::NotFound);
        }
        if let Some(displayname) = update.displayname {
            sqlx::query!(
                r#"UPDATE addressbook_shares SET displayname = ? WHERE (sharee, name) = (?, ?)"#,
                displayname,
                sharee,
                name
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }
        if let Some(color) = update.color {
            sqlx::query!(
                r#"UPDATE addressbook_shares SET color = ? WHERE (sharee, name) = (?, ?)"#,
                color,
                sharee,
                name
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }
        tx.commit().await.map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn set_principal_email(&self, principal: &str, email: Option<&str>) -> Result<(), Error> {
        set_principal_email(&self.db, principal, email).await
    }

    #[instrument]
    async fn principal_email(&self, principal: &str) -> Result<Option<String>, Error> {
        Ok(sqlx::query_scalar!(
            r#"SELECT email FROM principal_emails WHERE principal = ?"#,
            principal
        )
        .fetch_optional(&self.db)
        .await
        .map_err(crate::Error::from)?)
    }

    #[instrument]
    async fn resolve_email(&self, email: &str) -> Result<Option<String>, Error> {
        let email = email.trim().to_lowercase();
        Ok(sqlx::query_scalar!(
            r#"SELECT principal FROM principal_emails WHERE email = ?"#,
            email
        )
        .fetch_optional(&self.db)
        .await
        .map_err(crate::Error::from)?)
    }

    #[instrument]
    async fn get_visibility(&self, owner: &str, cal_id: &str) -> Result<Policy, Error> {
        let mut conn = self.db.acquire().await.map_err(crate::Error::from)?;
        Self::_get_visibility(&mut conn, owner, cal_id).await
    }

    #[instrument]
    async fn set_visibility(&self, owner: &str, cal_id: &str, policy: Policy) -> Result<(), Error> {
        let policy =
            Policy::new(policy.private, policy.confidential).ok_or(Error::InvalidVisibility)?;
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        let calendar = sqlx::query!(
            r#"SELECT push_topic FROM calendars
                WHERE (principal, id) = (?, ?) AND deleted_at IS NULL"#,
            owner,
            cal_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::Error::from)?
        .ok_or(Error::NotFound)?;
        let old = Self::_get_visibility(&mut tx, owner, cal_id).await?;
        if old == policy {
            return Ok(());
        }
        sqlx::query!(
            r#"INSERT INTO calendar_visibility (principal, cal_id, private, confidential) VALUES (?, ?, ?, ?)
                ON CONFLICT(principal, cal_id) DO UPDATE SET (private, confidential) = (excluded.private, excluded.confidential)"#,
            owner,
            cal_id,
            policy.private,
            policy.confidential
        )
        .execute(&mut *tx)
        .await
        .map_err(crate::Error::from)?;

        // Every object whose view changes is logged, shared now or not: a share ended and
        // created again has the same name, and its sharee may sync with a token from before.
        // Only objects with a CLASS other than PUBLIC are parsed, the text test is cheap.
        let objects = sqlx::query!(
            r#"SELECT id, ics FROM calendarobjects
                WHERE (principal, cal_id) = (?, ?) AND deleted_at IS NULL"#,
            owner,
            cal_id
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::Error::from)?;
        let changed: Vec<String> = objects
            .into_iter()
            .filter(|object| {
                visibility::may_depend_on_policy(&object.ics) && {
                    let classes = Classes::of(&object.ics);
                    classes.level(old) != classes.level(policy)
                }
            })
            .map(|object| object.id)
            .collect();
        let mut sync_token = None;
        for object_id in &changed {
            sync_token = Some(
                SqliteCalendarStore::log_object_operation(
                    &mut tx,
                    owner,
                    cal_id,
                    object_id,
                    ChangeOperation::Add,
                )
                .await?,
            );
        }
        tx.commit().await.map_err(crate::Error::from)?;
        if let Some(sync_token) = sync_token {
            self.notify(calendar.push_topic, sync_token);
        }
        Ok(())
    }

    #[instrument(skip(object))]
    async fn put_copy_object(
        &self,
        owner: &str,
        cal_id: &str,
        sharee: &str,
        object_id: &str,
        object: CalendarObject,
        preconditions: Preconditions<'_>,
    ) -> Result<Option<String>, Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        Self::check_calendar_write(&mut tx, owner, cal_id, sharee).await?;
        let topic = Self::writable_calendar(&mut tx, owner, cal_id).await?;
        let policy = Self::_get_visibility(&mut tx, owner, cal_id).await?;
        let row = sqlx::query!(
            r#"SELECT uid, ics, deleted_at IS NOT NULL AS "deleted!: bool" FROM calendarobjects
                WHERE (principal, cal_id, id) = (?, ?, ?)"#,
            owner,
            cal_id,
            object_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::Error::from)?;

        let stored = match row {
            // The owner's trash bin is never overwritten nor revived
            Some(row) if row.deleted => return Err(CopyRule::TrashHref.into()),
            Some(row) => {
                let stored = CalendarObject::from_ics(row.ics)?;
                if visibility::level_of(&stored, policy) < FULL {
                    return Err(CopyRule::Level.into());
                }
                let seen = visibility::view(&stored, FULL);
                preconditions.check(seen.as_ref().map(CalendarObject::get_etag).as_deref())?;
                if object.get_uid() != row.uid {
                    return Err(CopyRule::UidChanged.into());
                }
                Some(stored)
            }
            None => {
                preconditions.check(None)?;
                let uid = object.get_uid();
                let taken = sqlx::query_scalar!(
                    r#"SELECT EXISTS(SELECT 1 FROM calendarobjects WHERE (principal, cal_id, uid) = (?, ?, ?)) AS "taken!: bool""#,
                    owner,
                    cal_id,
                    uid
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(crate::Error::from)?;
                if taken {
                    return Err(CopyRule::ForeignUid.into());
                }
                None
            }
        };

        // Alarms are the owner's: kept from what is stored, never taken from the sharee
        let merged = visibility::with_stored_alarms(&object, stored.as_ref()).ok_or_else(|| {
            Error::Other(
                std::io::Error::other(format!("calendar object {object_id} cannot be merged"))
                    .into(),
            )
        })?;
        let ObjectColumns {
            uid,
            ics,
            first_occurence,
            last_occurence,
            etag,
            object_type,
        } = ObjectColumns::of(&merged);
        let written = if stored.is_some() {
            sqlx::query!(
                r#"UPDATE calendarobjects
                    SET uid = ?, ics = ?, first_occurence = date(?), last_occurence = date(?), etag = ?, object_type = ?, updated_at = datetime()
                    WHERE (principal, cal_id, id) = (?, ?, ?) AND deleted_at IS NULL"#,
                uid,
                ics,
                first_occurence,
                last_occurence,
                etag,
                object_type,
                owner,
                cal_id,
                object_id
            )
            .execute(&mut *tx)
            .await
        } else {
            sqlx::query!(
                r#"INSERT INTO calendarobjects (principal, cal_id, id, uid, ics, first_occurence, last_occurence, etag, object_type)
                    VALUES (?, ?, ?, ?, ?, date(?), date(?), ?, ?)"#,
                owner,
                cal_id,
                object_id,
                uid,
                ics,
                first_occurence,
                last_occurence,
                etag,
                object_type
            )
            .execute(&mut *tx)
            .await
        };
        match written.map_err(crate::Error::from).map_err(Error::from) {
            // The checks above make this impossible; the database stays the second guard
            Err(Error::AlreadyExists) => return Err(CopyRule::ForeignUid.into()),
            Err(err) => return Err(err),
            Ok(_) => {}
        }
        let sync_token = SqliteCalendarStore::log_object_operation(
            &mut tx,
            owner,
            cal_id,
            object_id,
            ChangeOperation::Add,
        )
        .await?;
        tx.commit().await.map_err(crate::Error::from)?;
        self.notify(topic, sync_token);

        let seen = visibility::view_for(&merged, policy);
        Ok(seen
            .filter(|seen| seen.get_ics() == object.get_ics())
            .map(|seen| seen.get_etag()))
    }

    #[instrument]
    async fn delete_copy_object(
        &self,
        owner: &str,
        cal_id: &str,
        sharee: &str,
        object_id: &str,
        preconditions: Preconditions<'_>,
    ) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        Self::check_calendar_write(&mut tx, owner, cal_id, sharee).await?;
        let topic = Self::writable_calendar(&mut tx, owner, cal_id).await?;
        let policy = Self::_get_visibility(&mut tx, owner, cal_id).await?;
        let ics = sqlx::query_scalar!(
            r#"SELECT ics FROM calendarobjects
                WHERE (principal, cal_id, id) = (?, ?, ?) AND deleted_at IS NULL"#,
            owner,
            cal_id,
            object_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::Error::from)?
        .ok_or(Error::NotFound)?;
        let stored = CalendarObject::from_ics(ics)?;
        match visibility::level_of(&stored, policy) {
            HIDDEN => return Err(Error::NotFound),
            FULL => {}
            _ => return Err(CopyRule::Level.into()),
        }
        let seen = visibility::view(&stored, FULL);
        preconditions.check(seen.as_ref().map(CalendarObject::get_etag).as_deref())?;
        // Always into the owner's trash bin
        sqlx::query!(
            r#"UPDATE calendarobjects SET deleted_at = datetime(), updated_at = datetime()
                WHERE (principal, cal_id, id) = (?, ?, ?)"#,
            owner,
            cal_id,
            object_id
        )
        .execute(&mut *tx)
        .await
        .map_err(crate::Error::from)?;
        let sync_token = SqliteCalendarStore::log_object_operation(
            &mut tx,
            owner,
            cal_id,
            object_id,
            ChangeOperation::Delete,
        )
        .await?;
        tx.commit().await.map_err(crate::Error::from)?;
        self.notify(topic, sync_token);
        Ok(())
    }

    #[instrument(skip(object))]
    async fn put_copy_address_object(
        &self,
        owner: &str,
        addressbook_id: &str,
        sharee: &str,
        object_id: &str,
        object: AddressObject,
        preconditions: Preconditions<'_>,
    ) -> Result<Option<String>, Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        Self::check_addressbook_write(&mut tx, owner, addressbook_id, sharee).await?;
        let topic = Self::live_addressbook(&mut tx, owner, addressbook_id).await?;
        let row = sqlx::query!(
            r#"SELECT vcf, deleted_at IS NOT NULL AS "deleted!: bool" FROM addressobjects
                WHERE (principal, addressbook_id, id) = (?, ?, ?)"#,
            owner,
            addressbook_id,
            object_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::Error::from)?;
        let vcf = object.get_vcf();
        match row {
            Some(row) if row.deleted => return Err(CopyRule::TrashHref.into()),
            Some(row) => {
                let stored = AddressObject::from_vcf(row.vcf)?;
                preconditions.check(Some(&stored.get_etag()))?;
                sqlx::query!(
                    r#"UPDATE addressobjects SET vcf = ?, updated_at = datetime()
                        WHERE (principal, addressbook_id, id) = (?, ?, ?) AND deleted_at IS NULL"#,
                    vcf,
                    owner,
                    addressbook_id,
                    object_id
                )
                .execute(&mut *tx)
                .await
                .map_err(crate::Error::from)?;
            }
            None => {
                preconditions.check(None)?;
                let inserted = sqlx::query!(
                    r#"INSERT INTO addressobjects (principal, addressbook_id, id, vcf) VALUES (?, ?, ?, ?)"#,
                    owner,
                    addressbook_id,
                    object_id,
                    vcf
                )
                .execute(&mut *tx)
                .await
                .map_err(crate::Error::from)
                .map_err(Error::from);
                match inserted {
                    Err(Error::AlreadyExists) => return Err(CopyRule::TrashHref.into()),
                    Err(err) => return Err(err),
                    Ok(_) => {}
                }
            }
        }
        let sync_token = SqliteAddressbookStore::log_object_operation(
            &mut tx,
            owner,
            addressbook_id,
            object_id,
            ChangeOperation::Add,
        )
        .await?;
        tx.commit().await.map_err(crate::Error::from)?;
        self.notify(topic, sync_token);
        Ok(Some(object.get_etag()))
    }

    #[instrument]
    async fn delete_copy_address_object(
        &self,
        owner: &str,
        addressbook_id: &str,
        sharee: &str,
        object_id: &str,
        preconditions: Preconditions<'_>,
    ) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        Self::check_addressbook_write(&mut tx, owner, addressbook_id, sharee).await?;
        let topic = Self::live_addressbook(&mut tx, owner, addressbook_id).await?;
        let vcf = sqlx::query_scalar!(
            r#"SELECT vcf FROM addressobjects
                WHERE (principal, addressbook_id, id) = (?, ?, ?) AND deleted_at IS NULL"#,
            owner,
            addressbook_id,
            object_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::Error::from)?
        .ok_or(Error::NotFound)?;
        let stored = AddressObject::from_vcf(vcf)?;
        preconditions.check(Some(&stored.get_etag()))?;
        sqlx::query!(
            r#"UPDATE addressobjects SET deleted_at = datetime(), updated_at = datetime()
                WHERE (principal, addressbook_id, id) = (?, ?, ?)"#,
            owner,
            addressbook_id,
            object_id
        )
        .execute(&mut *tx)
        .await
        .map_err(crate::Error::from)?;
        let sync_token = SqliteAddressbookStore::log_object_operation(
            &mut tx,
            owner,
            addressbook_id,
            object_id,
            ChangeOperation::Delete,
        )
        .await?;
        tx.commit().await.map_err(crate::Error::from)?;
        self.notify(topic, sync_token);
        Ok(())
    }
}

impl SqliteShareStore {
    fn notify(&self, topic: String, sync_token: String) {
        if let Err(err) = self.sender.try_send(CollectionOperation {
            topic,
            data: CollectionOperationInfo::Content { sync_token },
        }) {
            error_span!(
                "Error trying to send update notification:",
                err = format!("{err}"),
            );
        }
    }

    async fn _get_visibility(
        conn: &mut SqliteConnection,
        owner: &str,
        cal_id: &str,
    ) -> Result<Policy, Error> {
        let row = sqlx::query!(
            r#"SELECT private AS "private: u8", confidential AS "confidential: u8"
                FROM calendar_visibility WHERE (principal, cal_id) = (?, ?)"#,
            owner,
            cal_id
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(crate::Error::from)?;
        Ok(row
            .and_then(|row| Policy::new(row.private, row.confidential))
            .unwrap_or_default())
    }

    /// That `sharee` still has write access to the calendar, checked in the transaction of the
    /// write: [`Error::NotFound`] if the share is gone, [`Error::Forbidden`] if it only reads
    async fn check_calendar_write(
        tx: &mut Transaction<'_, Sqlite>,
        owner: &str,
        cal_id: &str,
        sharee: &str,
    ) -> Result<(), Error> {
        let access = sqlx::query_scalar!(
            r#"SELECT access FROM calendar_shares WHERE (owner, cal_id, sharee) = (?, ?, ?)"#,
            owner,
            cal_id,
            sharee
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(crate::Error::from)?
        .ok_or(Error::NotFound)?;
        match access.parse()? {
            Access::Write => Ok(()),
            Access::Read => Err(Error::Forbidden),
        }
    }

    /// See [`Self::check_calendar_write`]
    async fn check_addressbook_write(
        tx: &mut Transaction<'_, Sqlite>,
        owner: &str,
        addressbook_id: &str,
        sharee: &str,
    ) -> Result<(), Error> {
        let access = sqlx::query_scalar!(
            r#"SELECT access FROM addressbook_shares WHERE (owner, addressbook_id, sharee) = (?, ?, ?)"#,
            owner,
            addressbook_id,
            sharee
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(crate::Error::from)?
        .ok_or(Error::NotFound)?;
        match access.parse()? {
            Access::Write => Ok(()),
            Access::Read => Err(Error::Forbidden),
        }
    }

    /// The push topic of a live calendar that takes objects
    async fn writable_calendar(
        tx: &mut Transaction<'_, Sqlite>,
        owner: &str,
        cal_id: &str,
    ) -> Result<String, Error> {
        let calendar = sqlx::query!(
            r#"SELECT push_topic, subscription_url IS NOT NULL AS "subscription!: bool"
                FROM calendars WHERE (principal, id) = (?, ?) AND deleted_at IS NULL"#,
            owner,
            cal_id
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(crate::Error::from)?
        .ok_or(Error::NotFound)?;
        if calendar.subscription {
            return Err(Error::ReadOnly);
        }
        Ok(calendar.push_topic)
    }

    /// The push topic of a live addressbook
    async fn live_addressbook(
        tx: &mut Transaction<'_, Sqlite>,
        owner: &str,
        addressbook_id: &str,
    ) -> Result<String, Error> {
        sqlx::query_scalar!(
            r#"SELECT push_topic FROM addressbooks WHERE (principal, id) = (?, ?) AND deleted_at IS NULL"#,
            owner,
            addressbook_id
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(crate::Error::from)?
        .ok_or(Error::NotFound)
    }
}
