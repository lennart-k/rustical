use actix_web::ResponseError;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait UserStore: 'static {
    type Error: ResponseError;

    async fn user_exists(&self, id: &str) -> Result<bool, Self::Error>;
    async fn insert_user(&self, id: &str) -> Result<(), Self::Error>;
}
