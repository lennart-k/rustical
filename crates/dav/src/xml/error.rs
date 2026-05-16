use rustical_xml::{XmlRootTag, XmlSerialize};

#[derive(XmlSerialize, XmlRootTag)]
#[xml(ns = "crate::namespace::NS_DAV", root = "error")]
#[xml(ns_prefix(crate::namespace::NS_DAV = "",))]
pub struct ErrorElement<'t, T: XmlSerialize>(#[xml(ty = "untagged")] pub &'t T);

#[cfg(test)]
mod tests {
    use super::ErrorElement;
    use rustical_xml::{XmlSerialize, XmlSerializeRoot};

    #[derive(XmlSerialize, Default)]
    enum Error {
        #[default]
        UnfortunateError,
    }

    #[test]
    fn test_serialize_resourcetype() {
        let out = ErrorElement(&Error::UnfortunateError)
            .serialize_to_string()
            .unwrap();
        assert_eq!(
            out,
            r#"<?xml version="1.0" encoding="utf-8"?>
<error xmlns="DAV:">
    <unfortunate-error/>
</error>"#
        );
    }
}
