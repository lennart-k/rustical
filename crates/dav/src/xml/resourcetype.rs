use rustical_xml::XmlSerialize;

#[derive(Debug, Clone, PartialEq, XmlSerialize)]
pub struct Resourcetype(#[xml(flatten, ty = "untagged")] pub &'static [ResourcetypeInner]);

#[derive(Debug, Clone, PartialEq, XmlSerialize)]
pub struct ResourcetypeInner(
    #[xml(ty = "namespace")] pub quick_xml::name::Namespace<'static>,
    #[xml(ty = "tag_name")] pub &'static str,
);

#[cfg(test)]
mod tests {
    use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

    use super::{Resourcetype, ResourcetypeInner};

    #[derive(XmlSerialize, XmlRootTag)]
    #[xml(root = b"document")]
    struct Document {
        resourcetype: Resourcetype,
    }

    #[test]
    fn test_serialize_resourcetype() {
        let mut buf = Vec::new();
        let mut writer = quick_xml::Writer::new(&mut buf);
        Document {
            resourcetype: Resourcetype(&[
                ResourcetypeInner(crate::namespace::NS_DAV, "displayname"),
                ResourcetypeInner(crate::namespace::NS_CALENDARSERVER, "calendar-color"),
            ]),
        }
        .serialize_root(&mut writer)
        .unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(
            out,
            "<document><resourcetype><displayname xmlns=\"DAV:\"/><calendar-color xmlns=\"http://calendarserver.org/ns/\"/></resourcetype></document>"
        )
    }
}
