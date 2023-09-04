use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub name: Option<String>,
}

impl User {}

#[async_trait]
pub trait UserStore: Send + Sync + 'static {
    async fn get_user(&self, id: &str) -> Result<User>;
    async fn get_users(&self) -> Result<Vec<User>>;
}

pub struct TestUserStore {}

#[async_trait]
impl UserStore for TestUserStore {
    async fn get_user(&self, id: &str) -> Result<User> {
        if id != "test" {
            return Err(anyhow!("asd"));
        }
        Ok(User {
            id: "test".to_string(),
            name: Some("test".to_string()),
        })
    }

    async fn get_users(&self) -> Result<Vec<User>> {
        Ok(vec![User {
            id: "test".to_string(),
            name: Some("test".to_string()),
        }])
    }
}
