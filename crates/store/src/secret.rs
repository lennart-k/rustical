use derive_more::{AsRef, From};
use serde::{Deserialize, Serialize};

/// Wrapper type to prevent secrets from accidentally getting leaked into traces
#[derive(From, Clone, Deserialize, Serialize, AsRef)]
pub struct Secret<T>(pub T);

impl<T> Secret<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret")
    }
}
