use crate::{auth::User, error::Error};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{AuthenticationProvider, UserStore};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct StaticUserStoreConfig {
    pub users: Vec<User>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StaticUserStore {
    pub users: HashMap<String, User>,
}

impl StaticUserStore {
    pub fn new(config: StaticUserStoreConfig) -> Self {
        Self {
            users: HashMap::from_iter(config.users.into_iter().map(|user| (user.id.clone(), user))),
        }
    }
}

#[async_trait]
impl UserStore for StaticUserStore {
    async fn get_user(&self, id: &str) -> Result<Option<User>, crate::Error> {
        Ok(self.users.get(id).cloned())
    }
}

#[async_trait]
impl AuthenticationProvider for StaticUserStore {
    async fn validate_user_token(&self, user_id: &str, token: &str) -> Result<Option<User>, Error> {
        let user: User = match self.get_user(user_id).await? {
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
