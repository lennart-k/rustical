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
        // Hello(usize),
        // Unit,
    }

    let out = Document {
        prop: Prop::Test("asd".to_owned()),
    }
    .serialize_to_string()
    .unwrap();
    assert_eq!(
        out,
        r#"<?xml version="1.0" encoding="utf-8"?>
<propfind>
    <prop>
        <test>asd</test>
    </prop>
</propfind>"#
    );
}
