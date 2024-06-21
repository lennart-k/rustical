pub mod multistatus;
pub mod tag_list;
pub mod tag_name;

pub use multistatus::MultistatusElement;
pub use tag_list::TagList;
pub use tag_name::TagName;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HrefElement {
    pub href: String,
}

impl HrefElement {
    pub fn new(href: String) -> Self {
        Self { href }
    }
}
