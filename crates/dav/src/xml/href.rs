use derive_more::From;
use http::Uri;
use rustical_xml::{XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, Debug, Clone, From, PartialEq, Eq)]
pub struct HrefElement {
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub href: Uri,
}

impl HrefElement {
    #[must_use]
    pub const fn new(href: Uri) -> Self {
        Self { href }
    }
}

#[cfg(test)]
mod tests {
    use super::HrefElement;
    use http::Uri;
    use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

    #[derive(XmlSerialize, XmlRootTag)]
    #[xml(root = "document")]
    struct Document {
        hello: HrefElement,
    }

    #[test]
    fn test_serialize_resourcetype() {
        let out = Document {
            hello: HrefElement::new(Uri::from_static("/okaywow")),
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
