use rustical_xml::{XmlRootTag, XmlSerialize};
use xml_derive::XmlDeserialize;

#[test]
fn test_struct_document() {
    #[derive(Debug, XmlRootTag, XmlSerialize, XmlDeserialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        child: Child,
    }

    #[derive(Debug, XmlDeserialize, XmlSerialize, PartialEq, Default)]
    struct Child {
        #[xml(ty = "text")]
        text: String,
    }

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);
    Document {
        child: Child {
            text: "asd".to_owned(),
        },
    }
    .serialize(Some(Document::root_tag()), &mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    dbg!(out);
}
