use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

#[test]
fn test_struct_value_tagged() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
    #[xml(root = b"propfind")]
    struct Document {
        prop: Prop,
    }

    #[derive(Debug, XmlSerialize, PartialEq)]
    enum Prop {
        Test(String),
        Hello(usize),
        Unit,
    }

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);
    Document {
        prop: Prop::Test("asd".to_owned()),
    }
    .serialize_root(&mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert_eq!(out, "<propfind><prop><test>asd</test></prop></propfind>");
}
