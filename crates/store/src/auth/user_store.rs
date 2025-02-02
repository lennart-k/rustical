use super::User;
use async_trait::async_trait;

#[async_trait]
pub trait UserStore: 'static {
    async fn get_user(&self, id: &str) -> Result<Option<User>, crate::Error>;
}
