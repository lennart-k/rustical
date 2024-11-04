pub mod multistatus;
mod resourcetype;
pub mod tag_list;
pub mod tag_name;

use derive_more::derive::From;
pub use multistatus::MultistatusElement;
pub use tag_list::TagList;
pub use tag_name::TagName;

pub use resourcetype::Resourcetype;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, From, PartialEq)]
pub struct HrefElement {
    pub href: String,
}

impl HrefElement {
    pub fn new(href: String) -> Self {
        Self { href }
    }
}
