use rustical_store::auth::Principal;

pub mod user;

/// Required by the base layout
pub trait DefaultLayoutData {
    fn get_user(&self) -> Option<&Principal>;
}
