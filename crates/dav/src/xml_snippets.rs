use serde::Serialize;

#[derive(Serialize)]
pub struct HrefElement {
    pub href: String,
}
impl HrefElement {
    pub fn new(href: String) -> Self {
        Self { href }
    }
}

#[derive(Serialize)]
pub struct TextNode(pub Option<String>);
