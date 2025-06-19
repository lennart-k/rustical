use async_trait::async_trait;
use derive_more::Constructor;
use password_hash::PasswordHasher;
use pbkdf2::{
    Params,
    password_hash::{SaltString, rand_core::OsRng},
};
use rustical_store::{
    Error, Secret,
    auth::{AppToken, AuthenticationProvider, Principal},
};
use sqlx::{SqlitePool, types::Json};
use tracing::instrument;

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
        Ok(Principal {
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

#[derive(Debug, Constructor)]
pub struct SqlitePrincipalStore {
    db: SqlitePool,
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
        // Would be cleaner to put this into a transaction but for now it will be fine
        if !overwrite && self.get_principal(&user.id).await?.is_some() {
            return Err(Error::AlreadyExists);
        }
        let principal_type = user.principal_type.as_str();
        let password = user.password.map(Secret::into_inner);
        sqlx::query!(
            r#"
            REPLACE INTO principals
            (id, displayname, principal_type, password_hash)
            VALUES (?, ?, ?, ?)
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
            r#"SELECT id, displayname AS name, token, created_at AS "created_at: _" FROM app_tokens WHERE principal = ?"#,
            principal
        )
        .fetch_all(&self.db)
        .await
        .map_err(crate::Error::from)?)
    }

    #[instrument(skip(token))]
    async fn validate_app_token(
        &self,
        user_id: &str,
        token: &str,
    ) -> Result<Option<Principal>, Error> {
        for app_token in &self.get_app_tokens(user_id).await? {
            if password_auth::verify_password(token, app_token.token.as_ref()).is_ok() {
                return self.get_principal(user_id).await;
            }
        }
        Ok(None)
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
        Ok(())
    }

    #[instrument(skip(password_input))]
    async fn validate_password(
        &self,
        user_id: &str,
        password_input: &str,
    ) -> Result<Option<Principal>, Error> {
        let user: Principal = match self.get_principal(user_id).await? {
            Some(user) => user,
            None => return Ok(None),
        };
        let password = match &user.password {
            Some(password) => password,
            None => return Ok(None),
        };

        if password_auth::verify_password(password_input, password.as_ref()).is_ok() {
            return Ok(Some(user));
        }
        Ok(None)
    }

    #[instrument(skip(token))]
    async fn add_app_token(
        &self,
        user_id: &str,
        name: String,
        token: String,
    ) -> Result<String, Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let salt = SaltString::generate(OsRng);
        let token_hash = pbkdf2::Pbkdf2
            .hash_password_customized(
                token.as_bytes(),
                None,
                None,
                Params {
                    rounds: 100,
                    ..Default::default()
                },
                &salt,
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
