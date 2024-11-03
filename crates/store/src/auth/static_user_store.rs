use crate::{auth::User, error::Error};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::AuthenticationProvider;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StaticUserStoreConfig {
    users: Vec<UserEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserEntry {
    #[serde(flatten)]
    pub user: User,
    #[serde(default)]
    pub app_tokens: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StaticUserStore {
    pub users: HashMap<String, UserEntry>,
}

impl StaticUserStore {
    pub fn new(config: StaticUserStoreConfig) -> Self {
        Self {
            users: HashMap::from_iter(
                config
                    .users
                    .into_iter()
                    .map(|user_entry| (user_entry.user.id.clone(), user_entry)),
            ),
        }
    }
}

#[async_trait]
impl AuthenticationProvider for StaticUserStore {
    async fn validate_user_token(&self, user_id: &str, token: &str) -> Result<Option<User>, Error> {
        let user_entry: UserEntry = match self.users.get(user_id) {
            Some(user) => user.clone(),
            None => return Ok(None),
        };

        // Try app tokens first since they are cheaper to calculate
        // They can afford less iterations since they can be generated with high entropy
        for app_token in &user_entry.app_tokens {
            if password_auth::verify_password(token, app_token).is_ok() {
                return Ok(Some(user_entry.user));
            }
        }

        let password = match &user_entry.user.password {
            Some(password) => password,
            None => return Ok(None),
        };

        if password_auth::verify_password(token, password).is_ok() {
            return Ok(Some(user_entry.user));
        }

        Ok(None)
    }
}
