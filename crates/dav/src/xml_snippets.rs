use anyhow::Result;
use quick_xml::{events::attributes::Attribute, Writer};
use serde::ser::SerializeMap;
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

pub struct TagList(pub Vec<String>);

impl Serialize for TagList {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut el = serializer.serialize_map(Some(self.0.len()))?;
        for tag in &self.0 {
            el.serialize_entry(&tag, &())?;
        }
        el.end()
    }
}

pub fn generate_multistatus<'a, F, A>(namespaces: A, closure: F) -> Result<String>
where
    F: FnOnce(&mut Writer<&mut Vec<u8>>) -> Result<(), quick_xml::Error>,
    A: IntoIterator,
    A::Item: Into<Attribute<'a>>,
{
    let mut output_buffer = Vec::new();
    let mut writer = Writer::new_with_indent(&mut output_buffer, b' ', 2);
    writer
        .create_element("multistatus")
        .with_attributes(namespaces)
        .write_inner_content(closure)?;

    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{}",
        std::str::from_utf8(&output_buffer)?
    ))
}
