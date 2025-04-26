use super::{AuthenticationProvider, user::AppToken};
use crate::{auth::User, error::Error};
use anyhow::anyhow;
use async_trait::async_trait;
use password_hash::PasswordHasher;
use pbkdf2::{
    Params,
    password_hash::{self, SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, ops::Deref};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct TomlDataModel {
    principals: Vec<User>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TomlUserStoreConfig {
    pub path: String,
}

#[derive(Debug)]
pub struct TomlPrincipalStore {
    pub principals: RwLock<HashMap<String, User>>,
    config: TomlUserStoreConfig,
}

#[derive(thiserror::Error, Debug)]
pub enum TomlStoreError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Error parsing users toml: {0}")]
    Toml(#[from] toml::de::Error),
}

impl TomlPrincipalStore {
    pub fn new(config: TomlUserStoreConfig) -> Result<Self, TomlStoreError> {
        let TomlDataModel { principals } = toml::from_str(&fs::read_to_string(&config.path)?)?;
        Ok(Self {
            principals: RwLock::new(HashMap::from_iter(
                principals.into_iter().map(|user| (user.id.clone(), user)),
            )),
            config,
        })
    }

    fn save(&self, principals: &HashMap<String, User>) -> Result<(), Error> {
        let out = toml::to_string_pretty(&TomlDataModel {
            principals: principals
                .iter()
                .map(|(_, value)| value.to_owned())
                .collect(),
        })
        .map_err(|_| anyhow!("Error saving principal database"))?;
        fs::write(&self.config.path, out)?;
        Ok(())
    }
}

#[async_trait]
impl AuthenticationProvider for TomlPrincipalStore {
    async fn get_principals(&self) -> Result<Vec<User>, crate::Error> {
        Ok(self.principals.read().await.values().cloned().collect())
    }

    async fn get_principal(&self, id: &str) -> Result<Option<User>, crate::Error> {
        Ok(self.principals.read().await.get(id).cloned())
    }

    async fn insert_principal(&self, user: User, overwrite: bool) -> Result<(), crate::Error> {
        let mut principals = self.principals.write().await;
        if !overwrite && principals.contains_key(&user.id) {
            return Err(Error::AlreadyExists);
        }
        principals.insert(user.id.clone(), user);
        self.save(principals.deref())?;
        Ok(())
    }

    async fn remove_principal(&self, id: &str) -> Result<(), crate::Error> {
        let mut principals = self.principals.write().await;
        principals.remove(id);
        self.save(principals.deref())?;
        Ok(())
    }

    async fn validate_password(
        &self,
        user_id: &str,
        password_input: &str,
    ) -> Result<Option<User>, Error> {
        let user: User = match self.get_principal(user_id).await? {
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

    async fn validate_app_token(&self, user_id: &str, token: &str) -> Result<Option<User>, Error> {
        let user: User = match self.get_principal(user_id).await? {
            Some(user) => user,
            None => return Ok(None),
        };

        for app_token in &user.app_tokens {
            if password_auth::verify_password(token, app_token.token.as_ref()).is_ok() {
                return Ok(Some(user));
            }
        }
        Ok(None)
    }

    /// Returns an identifier for the new app token
    async fn add_app_token(
        &self,
        user_id: &str,
        name: String,
        token: String,
    ) -> Result<String, Error> {
        let mut principals = self.principals.write().await;
        if let Some(principal) = principals.get_mut(user_id) {
            let id = uuid::Uuid::new_v4().to_string();
            let salt = SaltString::generate(OsRng);
            let token_hash = pbkdf2::Pbkdf2
                .hash_password_customized(
                    token.as_bytes(),
                    None,
                    None,
                    Params {
                        rounds: 1000,
                        ..Default::default()
                    },
                    &salt,
                )
                .map_err(|_| Error::PasswordHash)?
                .to_string();
            principal.app_tokens.push(AppToken {
                name,
                token: token_hash.into(),
                created_at: Some(chrono::Utc::now()),
                id: id.clone(),
            });
            self.save(principals.deref())?;
            Ok(id)
        } else {
            Err(Error::NotFound)
        }
    }

    async fn remove_app_token(&self, user_id: &str, token_id: &str) -> Result<(), Error> {
        let mut principals = self.principals.write().await;
        if let Some(principal) = principals.get_mut(user_id) {
            principal
                .app_tokens
                .retain(|AppToken { id, .. }| token_id != id);
            self.save(principals.deref())?;
        }
        Ok(())
    }
}
