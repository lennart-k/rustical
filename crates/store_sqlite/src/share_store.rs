use crate::BEGIN_IMMEDIATE;
use async_trait::async_trait;
use derive_more::Constructor;
use rustical_store::{
    Access, AddressbookShare, CalendarShare, CollectionOperation, Error, ShareRejection,
    ShareStore, ShareViewUpdate,
};
use sqlx::types::chrono::NaiveDateTime;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::mpsc::Sender;
use tracing::{instrument, warn};

#[cfg(test)]
mod tests;

/// Calendar ids with this prefix belong to the birthday calendars of addressbooks
const BIRTHDAYS_PREFIX: &str = "_birthdays_";

#[derive(Debug, Clone, Constructor)]
pub struct SqliteShareStore {
    db: SqlitePool,
    // Push notifications for changes made through shares
    #[allow(dead_code)]
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
        let mut tx = self
            .db
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
}
