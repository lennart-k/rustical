use quick_xml::Writer;
use quick_xml::name::Namespace;
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};
use std::collections::HashMap;
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

    Document {
        child: Child {
            text: "asd".to_owned(),
        },
    }
    .serialize_to_string()
    .unwrap();
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

    Document {
        name: "okay".to_owned(),
        child: Child {
            text: "asd".to_owned(),
        },
    }
    .serialize_to_string()
    .unwrap();
}

#[test]
fn test_struct_value_tagged() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        href: String,
        num: usize,
    }

    let out = Document {
        href: "okay".to_owned(),
        num: 123,
    }
    .serialize_to_string()
    .unwrap();
    assert_eq!(
        out,
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<document><href>okay</href><num>123</num></document>"
    );
}

#[test]
fn test_struct_value_untagged() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(ty = "untagged")]
        href: String,
    }

    let out = Document {
        href: "okays".to_owned(),
    }
    .serialize_to_string()
    .unwrap();
    assert_eq!(
        out,
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<document>okays</document>"
    );
}

#[test]
fn test_struct_vec() {
    #[derive(Debug, XmlRootTag, XmlSerialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(flatten)]
        href: Vec<String>,
    }

    let out = Document {
        href: vec!["okay".to_owned(), "wow".to_owned()],
    }
    .serialize_to_string()
    .unwrap();
    assert_eq!(
        out,
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<document><href>okay</href><href>wow</href></document>"
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

    fn serialize_href(
        val: &str,
        ns: Option<Namespace>,
        tag: Option<&[u8]>,
        namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut Writer<&mut Vec<u8>>,
    ) -> std::io::Result<()> {
        val.to_uppercase().serialize(ns, tag, namespaces, writer)
    }

    let out = Document {
        href: "okay".to_owned(),
    }
    .serialize_to_string()
    .unwrap();
    assert_eq!(
        out,
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<document><href>OKAY</href></document>"
    );
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
    .serialize_to_string()
    .unwrap();
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

    Document {
        child: "hello!".to_string(),
    }
    .serialize_to_string()
    .unwrap();
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

    Document {
        child: Child("child".to_owned(), "Hello!".to_owned()),
    }
    .serialize_to_string()
    .unwrap();
}

#[test]
fn test_tuple_struct() {
    const NS: Namespace = quick_xml::name::Namespace(b"NS:TEST:");

    #[derive(Debug, XmlRootTag, XmlSerialize)]
    #[xml(root = b"document")]
    struct Document(#[xml(ns = "NS", rename = b"okay")] String);

    Document("hello!".to_string())
        .serialize_to_string()
        .unwrap();
}
