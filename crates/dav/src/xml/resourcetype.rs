use rustical_xml::XmlSerialize;

#[derive(Debug, Clone, PartialEq, Eq, XmlSerialize)]
pub struct Resourcetype(#[xml(flatten, ty = "untagged")] pub &'static [ResourcetypeInner]);

#[macro_export]
macro_rules! resourcetype {
    ($(($ns:expr, $name:expr)),* $(,)?) => {
        ::rustical_dav::xml::Resourcetype(&[
            $(::rustical_dav::xml::ResourcetypeInner(Some($ns), $name)),*
        ])
    };
}

pub use resourcetype;

#[derive(Debug, Clone, PartialEq, Eq, XmlSerialize)]
pub struct ResourcetypeInner(
    #[xml(ty = "namespace")] pub Option<quick_xml::name::Namespace<'static>>,
    #[xml(ty = "tag_name")] pub &'static str,
);

#[cfg(test)]
mod tests {
    use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

    use crate::namespace::{NS_CALENDARSERVER, NS_DAV};

    use super::{Resourcetype, ResourcetypeInner};

    #[derive(XmlSerialize, XmlRootTag)]
    #[xml(root = "document")]
    struct Document {
        resourcetype: Resourcetype,
    }

    #[test]
    fn test_serialize_resourcetype() {
        let out = Document {
            resourcetype: Resourcetype(&[
                ResourcetypeInner(Some(NS_DAV), "displayname"),
                ResourcetypeInner(Some(NS_CALENDARSERVER), "calendar-color"),
            ]),
        }
        .serialize_to_string()
        .unwrap();
        assert_eq!(
            out,
            r#"<?xml version="1.0" encoding="utf-8"?>
<document>
    <resourcetype>
        <displayname xmlns="DAV:"/>
        <calendar-color xmlns="http://calendarserver.org/ns/"/>
    </resourcetype>
</document>"#
        );
    }
}
