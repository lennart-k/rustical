use async_trait::async_trait;
use axum::response::IntoResponse;

#[async_trait]
pub trait UserStore: 'static + Send {
    type Error: IntoResponse;

    async fn user_exists(&self, id: &str) -> Result<bool, Self::Error>;
    async fn insert_user(&self, id: &str) -> Result<(), Self::Error>;
}
