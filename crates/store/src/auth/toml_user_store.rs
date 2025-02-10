use super::{user::AppToken, AuthenticationProvider};
use crate::{auth::User, error::Error};
use anyhow::anyhow;
use async_trait::async_trait;
use password_hash::PasswordHasher;
use pbkdf2::{
    password_hash::{self, rand_core::OsRng, SaltString},
    Params,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, ops::Deref};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct TomlDataModel {
    principals: Vec<User>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
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
    async fn get_principal(&self, id: &str) -> Result<Option<User>, crate::Error> {
        Ok(self.principals.read().await.get(id).cloned())
    }

    async fn validate_user_token(&self, user_id: &str, token: &str) -> Result<Option<User>, Error> {
        let user: User = match self.get_principal(user_id).await? {
            Some(user) => user,
            None => return Ok(None),
        };

        // Try app tokens first since they are cheaper to calculate
        // They can afford less iterations since they can be generated with high entropy
        for app_token in &user.app_tokens {
            if password_auth::verify_password(token, &app_token.token).is_ok() {
                return Ok(Some(user));
            }
        }

        let password = match &user.password {
            Some(password) => password,
            None => return Ok(None),
        };

        if password_auth::verify_password(token, password).is_ok() {
            return Ok(Some(user));
        }

        Ok(None)
    }

    async fn add_app_token(&self, user_id: &str, name: String, token: String) -> Result<(), Error> {
        let mut principals = self.principals.write().await;
        if let Some(principal) = principals.get_mut(user_id) {
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
                token: token_hash,
                created_at: Some(chrono::Utc::now()),
            });
            self.save(principals.deref())?;
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }
}
