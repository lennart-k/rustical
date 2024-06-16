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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextNode(pub Option<String>);
