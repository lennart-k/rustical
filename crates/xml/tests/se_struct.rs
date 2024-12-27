use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};
use xml_derive::XmlDeserialize;

#[test]
fn test_struct_document() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
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
    .serialize_root(&mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
}

#[test]
fn test_struct_untagged_attr() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(ty = "attr")]
        name: String,
        #[xml(ty = "untagged")]
        child: Child,
    }

    #[derive(Debug, XmlSerialize, PartialEq, Default)]
    struct Child {
        #[xml(ty = "attr")]
        text: String,
    }

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);
    Document {
        name: "okay".to_owned(),
        child: Child {
            text: "asd".to_owned(),
        },
    }
    .serialize_root(&mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
}

#[test]
fn test_struct_value_tagged() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        href: String,
        num: usize,
    }

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);
    Document {
        href: "okay".to_owned(),
        num: 123,
    }
    .serialize_root(&mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert_eq!(out, "<document><href>okay</href><num>123</num></document>");
}

#[test]
fn test_struct_value_untagged() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(ty = "untagged")]
        href: String,
    }

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);
    Document {
        href: "okays".to_owned(),
    }
    .serialize_root(&mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert_eq!(out, "<document>okays</document>");
}

#[test]
fn test_struct_vec() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(flatten)]
        href: Vec<String>,
    }

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);
    Document {
        href: vec!["okay".to_owned(), "wow".to_owned()],
    }
    .serialize_root(&mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert_eq!(
        out,
        "<document><href>okay</href><href>wow</href></document>"
    );
}
