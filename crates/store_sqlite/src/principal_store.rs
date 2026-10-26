use crate::BEGIN_IMMEDIATE;
use async_trait::async_trait;
use password_hash::{CustomizedPasswordHasher, phc::Salt};
use pbkdf2::Params;
use rand::rngs::SysRng;
use rustical_store::{
    Error, Secret,
    auth::{AppToken, AuthenticationProvider, Principal},
};
use sqlx::{SqlitePool, types::Json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant},
};
use tracing::instrument;

/// The use of one app token is written at most this often, so that a client which syncs
/// every few minutes does not cause a database write per request.
const TOUCH_INTERVAL: Duration = Duration::from_hours(1);

#[derive(Debug, Default, Clone)]
struct PrincipalRow {
    id: String,
    displayname: Option<String>,
    principal_type: String,
    password_hash: Option<String>,
    memberships: Option<Json<Vec<Option<String>>>>,
}

impl TryFrom<PrincipalRow> for Principal {
    type Error = Error;

    fn try_from(value: PrincipalRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            displayname: value.displayname,
            password: value.password_hash.map(Secret::from),
            principal_type: value.principal_type.as_str().try_into()?,
            memberships: value
                .memberships
                .map(|val| val.0)
                .unwrap_or_default()
                .into_iter()
                .flatten()
                .collect(),
        })
    }
}

#[derive(Clone)]
pub struct SqlitePrincipalStore {
    db: SqlitePool,
    /// When the use of each app token was last written, shared by all clones.
    touched: Arc<Mutex<HashMap<String, Instant>>>,
}

impl SqlitePrincipalStore {
    #[must_use]
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            touched: Arc::default(),
        }
    }
}

impl std::fmt::Debug for SqlitePrincipalStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqlitePrincipalStore")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl AuthenticationProvider for SqlitePrincipalStore {
    #[instrument]
    async fn get_principals(&self) -> Result<Vec<Principal>, Error> {
        let result: Result<Vec<Principal>, Error> = sqlx::query_as!(
            PrincipalRow,
            r#"
            SELECT id, displayname, principal_type, password_hash, json_group_array(member_of) AS "memberships: Json<Vec<Option<String>>>"
            FROM principals
            LEFT JOIN memberships ON principals.id == memberships.principal
            GROUP BY principals.id
        "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(crate::Error::from)?
        .into_iter()
        .map(Principal::try_from)
        .collect();
        Ok(result?)
    }

    #[instrument]
    async fn get_principal(&self, id: &str) -> Result<Option<Principal>, Error> {
        let row= sqlx::query_as!(
            PrincipalRow,
            r#"
            SELECT id, displayname, principal_type, password_hash, json_group_array(member_of) AS "memberships: Json<Vec<Option<String>>>"
            FROM (SELECT * FROM principals WHERE id = ?) AS principals
            LEFT JOIN memberships ON principals.id == memberships.principal
            GROUP BY principals.id
        "#,
            id
        )
            .fetch_optional(&self.db)
            .await
            .map_err(crate::Error::from)?
            .map(Principal::try_from);
        if let Some(row) = row {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    #[instrument]
    async fn remove_principal(&self, id: &str) -> Result<(), Error> {
        sqlx::query!(r#"DELETE FROM principals WHERE id = ?"#, id)
            .execute(&self.db)
            .await
            .map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn insert_principal(
        &self,
        user: Principal,
        overwrite: bool,
    ) -> Result<(), rustical_store::Error> {
        if user.id.contains(':') || user.id.contains('$') {
            return Err(rustical_store::Error::InvalidPrincipalId);
        }

        // Would be cleaner to put this into a transaction but for now it will be fine
        if !overwrite && self.get_principal(&user.id).await?.is_some() {
            return Err(Error::AlreadyExists);
        }
        let principal_type = user.principal_type.as_str();
        let password = user.password.map(Secret::into_inner);
        sqlx::query!(
            r#"
            INSERT INTO principals
            (id, displayname, principal_type, password_hash) VALUES (?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                (displayname, principal_type, password_hash)
                = (excluded.displayname, excluded.principal_type, excluded.password_hash)
        "#,
            user.id,
            user.displayname,
            principal_type,
            password
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn get_app_tokens(&self, principal: &str) -> Result<Vec<AppToken>, Error> {
        Ok(sqlx::query_as!(
            AppToken,
            r#"SELECT id, displayname AS name, token, created_at AS "created_at: _", last_used AS "last_used: _" FROM app_tokens WHERE principal = ?"#,
            principal
        )
        .fetch_all(&self.db)
        .await
        .map_err(crate::Error::from)?)
    }

    #[instrument]
    async fn remove_app_token(&self, user_id: &str, token_id: &str) -> Result<(), Error> {
        sqlx::query!(
            r#"DELETE FROM app_tokens WHERE (principal, id) = (?, ?)"#,
            user_id,
            token_id
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        self.touched
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(token_id);
        Ok(())
    }

    #[instrument]
    async fn touch_app_token(&self, user_id: &str, token_id: &str) -> Result<(), Error> {
        {
            // Claimed before the write: concurrent first requests write once, and a busy or
            // failing database costs one attempt per interval instead of one per request
            // (each attempt may wait for the busy timeout).
            let mut touched = self.touched.lock().unwrap_or_else(PoisonError::into_inner);
            if touched
                .get(token_id)
                .is_some_and(|at| at.elapsed() < TOUCH_INTERVAL)
            {
                return Ok(());
            }
            touched.insert(token_id.to_owned(), Instant::now());
        }
        sqlx::query!(
            r#"UPDATE app_tokens SET last_used = CURRENT_TIMESTAMP WHERE (principal, id) = (?, ?)"#,
            user_id,
            token_id
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument(skip(token))]
    async fn add_app_token(
        &self,
        user_id: &str,
        name: String,
        token: String,
    ) -> Result<String, Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let salt = Salt::try_from_rng(&mut SysRng).map_err(|err| Error::Other(err.into()))?;
        let token_hash = pbkdf2::Pbkdf2::SHA512
            .hash_password_with_params(
                token.as_bytes(),
                &salt,
                // The app token has a high entropy so we are quite safe from quessing attacks
                // Also if an attacker got access to the hashes they'd have already gotten
                // access to the whole database.
                Params::new(1000).expect("1000 rounds are valid"),
            )
            .map_err(|_| Error::PasswordHash)?
            .to_string();
        sqlx::query!(
            r#"
            INSERT INTO app_tokens
                (id, principal, token, displayname)
            VALUES (?, ?, ?, ?)
        "#,
            id,
            user_id,
            token_hash,
            name
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        Ok(id)
    }

    #[instrument]
    async fn add_membership(&self, principal: &str, member_of: &str) -> Result<(), Error> {
        sqlx::query!(
            r#"REPLACE INTO memberships (principal, member_of) VALUES (?, ?)"#,
            principal,
            member_of
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn remove_membership(&self, principal: &str, member_of: &str) -> Result<(), Error> {
        sqlx::query!(
            r#"DELETE FROM memberships WHERE (principal, member_of) = (?, ?)"#,
            principal,
            member_of
        )
        .execute(&self.db)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn clear_memberships(&self, group: &str) -> Result<Vec<String>, Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        let members = sqlx::query_scalar!(
            r#"DELETE FROM memberships WHERE member_of = ? RETURNING principal"#,
            group
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::Error::from)?;
        tx.commit().await.map_err(crate::Error::from)?;
        Ok(members)
    }

    #[instrument]
    async fn set_memberships(&self, group: &str, members: &[String]) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;
        for member in members {
            sqlx::query!(
                r#"INSERT OR IGNORE INTO memberships (principal, member_of)
                    SELECT ?1, ?2 WHERE EXISTS(SELECT 1 FROM principals WHERE id = ?1)"#,
                member,
                group
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
        }
        tx.commit().await.map_err(crate::Error::from)?;
        Ok(())
    }

    #[instrument]
    async fn list_members(&self, principal: &str) -> Result<Vec<String>, Error> {
        Ok(sqlx::query!(
            r#"SELECT principal FROM memberships WHERE member_of = ?"#,
            principal
        )
        .fetch_all(&self.db)
        .await
        .map_err(crate::Error::from)?
        .into_iter()
        .map(|record| record.principal)
        .collect())
    }
}
