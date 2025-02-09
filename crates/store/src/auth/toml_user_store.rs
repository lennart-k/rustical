use super::AuthenticationProvider;
use crate::{auth::User, error::Error};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io};
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
        })
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
            if password_auth::verify_password(token, app_token).is_ok() {
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
}
