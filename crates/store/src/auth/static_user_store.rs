use crate::{auth::User, error::Error};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::AuthenticationProvider;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StaticUserStoreConfig {
    users: Vec<User>,
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
impl AuthenticationProvider for StaticUserStore {
    async fn validate_user_token(&self, user_id: &str, token: &str) -> Result<Option<User>, Error> {
        let user: User = match self.users.get(user_id) {
            Some(user) => user.clone(),
            None => return Ok(None),
        };

        let password = match &user.password {
            Some(password) => password,
            None => return Ok(None),
        };

        Ok(password_auth::verify_password(token, password)
            .map(|()| user)
            .ok())
    }
}
