use derive_more::From;
use rustical_xml::{XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, Debug, Clone, From, PartialEq, Eq)]
pub struct HrefElement {
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub href: String,
}

impl HrefElement {
    #[must_use]
    pub const fn new(href: String) -> Self {
        Self { href }
    }
}

#[cfg(test)]
mod tests {
    use super::HrefElement;
    use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

    #[derive(XmlSerialize, XmlRootTag)]
    #[xml(root = "document")]
    struct Document {
        hello: HrefElement,
    }

    #[test]
    fn test_serialize_resourcetype() {
        let out = Document {
            hello: HrefElement::new("/okaywow".to_owned()),
        }
        .serialize_to_string()
        .unwrap();
        assert_eq!(
            out,
            r#"<?xml version="1.0" encoding="utf-8"?>
<document>
    <hello>
        <href xmlns="DAV:">/okaywow</href>
    </hello>
</document>"#
        );
    }
}
