use crate::{auth::User, error::Error};
use async_trait::async_trait;

#[async_trait]
pub trait UserStore: Send + Sync + 'static {
    async fn get_user(&self, id: &str) -> Result<Option<User>, Error>;
    async fn put_user(&self, user: User) -> Result<(), Error>;
}
