use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
};

use quick_xml::Writer;
use quick_xml::name::Namespace;
use rustical_xml::{XmlDocument, XmlRootTag, XmlSerialize, XmlSerializeRoot};
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

#[test]
fn test_struct_serialize_with() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(serialize_with = "serialize_href")]
        href: String,
    }

    fn serialize_href<W: ::std::io::Write>(
        val: &str,
        ns: Option<Namespace>,
        tag: Option<&[u8]>,
        namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut Writer<W>,
    ) -> std::io::Result<()> {
        val.to_uppercase().serialize(ns, tag, namespaces, writer)
    }

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);
    Document {
        href: "okay".to_owned(),
    }
    .serialize_root(&mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert_eq!(out, "<document><href>OKAY</href></document>");
}

#[test]
fn test_struct_tag_list() {
    #[derive(Debug, XmlRootTag, XmlSerialize, XmlDeserialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(ty = "untagged", flatten)]
        tags: Vec<Tag>,
    }

    #[derive(Debug, XmlSerialize, XmlDeserialize, PartialEq)]
    struct Tag {
        #[xml(ty = "tag_name")]
        name: String,
    }

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);
    Document {
        tags: vec![
            Tag {
                name: "hello".to_owned(),
            },
            Tag {
                name: "ok".to_owned(),
            },
            Tag {
                name: "wow".to_owned(),
            },
        ],
    }
    .serialize_root(&mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    dbg!(out);
}

#[test]
fn test_struct_ns() {
    const NS: Namespace = quick_xml::name::Namespace(b"NS:TEST:");

    #[derive(Debug, XmlRootTag, XmlSerialize)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(ns = "NS", rename = b"okay")]
        child: String,
    }

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);
    Document {
        child: "hello!".to_string(),
    }
    .serialize_root(&mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    dbg!(out);
}

#[test]
fn test_struct_tuple() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        child: Child,
    }

    #[derive(Debug, XmlSerialize, PartialEq, Default)]
    struct Child(#[xml(ty = "tag_name")] String, #[xml(ty = "text")] String);

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);

    Document {
        child: Child("child".to_owned(), "Hello!".to_owned()),
    }
    .serialize_root(&mut writer)
    .unwrap();
    let out = String::from_utf8(buf).unwrap();
    dbg!(out);
}

#[test]
fn test_tuple_struct() {
    const NS: Namespace = quick_xml::name::Namespace(b"NS:TEST:");

    #[derive(Debug, XmlRootTag, XmlSerialize)]
    #[xml(root = b"document")]
    struct Document(#[xml(ns = "NS", rename = b"okay")] String);

    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);
    Document("hello!".to_string())
        .serialize_root(&mut writer)
        .unwrap();
    let out = String::from_utf8(buf).unwrap();
    dbg!(out);
}
