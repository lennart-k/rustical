use quick_xml::name::Namespace;
use rustical_xml::de::XmlDocument;
use rustical_xml::{Unparsed, XmlDeserialize, XmlRootTag};
use std::collections::HashSet;

#[test]
fn test_struct_text_field() {
    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(ty = "text")]
        text: String,
        #[xml(ty = "text")]
        text2: String,
    }

    let doc = Document::parse_str(r#"<document>Hello!</document>"#).unwrap();
    assert_eq!(
        doc,
        Document {
            text: "Hello!".to_owned(),
            text2: "Hello!".to_owned()
        }
    );
}

#[test]
fn test_struct_document() {
    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        child: Child,
    }

    #[derive(Debug, XmlDeserialize, PartialEq, Default)]
    struct Child {
        #[xml(ty = "text")]
        text: String,
    }

    let doc = Document::parse_str(r#"<document><child>Hello!</child></document>"#).unwrap();
    assert_eq!(
        doc,
        Document {
            child: Child {
                text: "Hello!".to_owned()
            }
        }
    );
}

#[test]
fn test_struct_rename_field() {
    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(rename = b"ok-wow")]
        child: Child,
    }

    #[derive(Debug, XmlDeserialize, PartialEq, Default)]
    struct Child {
        #[xml(ty = "text")]
        text: String,
    }

    let doc = Document::parse_str(r#"<document><ok-wow>Hello!</ok-wow></document>"#).unwrap();
    assert_eq!(
        doc,
        Document {
            child: Child {
                text: "Hello!".to_owned()
            },
        }
    );
}

#[test]
fn test_struct_optional_field() {
    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        child: Option<Child>,
    }

    #[derive(Debug, XmlDeserialize, PartialEq, Default)]
    struct Child;

    let doc = Document::parse_str(r#"<document><child></child></document>"#).unwrap();
    assert_eq!(doc, Document { child: Some(Child) });

    let doc = Document::parse_str(r#"<document></document>"#).unwrap();
    assert_eq!(doc, Document { child: None });
}

#[test]
fn test_struct_vec() {
    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(rename = b"child", flatten)]
        children: Vec<Child>,
    }

    #[derive(Debug, XmlDeserialize, PartialEq, Default)]
    struct Child;

    let doc = Document::parse_str(
        r#"
        <document>
            <child />
            <child />
        </document>"#,
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            children: vec![Child, Child]
        }
    );
}

#[test]
fn test_struct_set() {
    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(rename = b"child", flatten)]
        children: HashSet<Child>,
    }

    #[derive(Debug, XmlDeserialize, PartialEq, Default, Eq, Hash)]
    struct Child;

    let doc = Document::parse_str(
        r#"
        <document>
            <child />
            <child />
        </document>"#,
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            children: HashSet::from_iter(vec![Child].into_iter())
        }
    );
}

#[test]
fn test_struct_ns() {
    const NS_HELLO: Namespace = Namespace(b"hello");

    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"document", ns_strict)]
    struct Document {
        #[xml(ns = "NS_HELLO")]
        child: (),
    }

    let doc = Document::parse_str(r#"<document><child xmlns="hello" /></document>"#).unwrap();
    assert_eq!(doc, Document { child: () });
}

#[test]
fn test_struct_attr() {
    const NS_HELLO: Namespace = Namespace(b"hello");

    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"document", ns_strict)]
    struct Document {
        #[xml(ns = "NS_HELLO")]
        child: (),
        #[xml(ty = "attr", default = "Default::default")]
        test: String,
        #[xml(ty = "attr")]
        number: usize,
    }

    let doc = Document::parse_str(
        r#"<document test="hello!" number="2"><child xmlns="hello" /></document>"#,
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            child: (),
            test: "hello!".to_owned(),
            number: 2
        }
    );
}

#[test]
fn test_struct_generics() {
    #[derive(XmlDeserialize, XmlRootTag)]
    #[xml(root = b"document", ns_strict)]
    struct Document<T: XmlDeserialize> {
        child: T,
    }

    let doc = Document::<Unparsed>::parse_str(
        r#"
         <document>
             <child>
                 Hello! <h1>Nice</h1>
             </child>
         </document>
     "#,
    )
    .unwrap();
}

#[test]
fn test_struct_unparsed() {
    #[derive(XmlDeserialize, XmlRootTag)]
    #[xml(root = b"document", ns_strict)]
    struct Document {
        child: Unparsed,
    }

    let doc = Document::parse_str(
        r#"
         <document>
             <child>
                 Hello! <h1>Nice</h1>
             </child>
         </document>
     "#,
    )
    .unwrap();
}

#[test]
fn test_xml_values() {
    #[derive(XmlDeserialize, XmlRootTag, PartialEq, Debug)]
    #[xml(root = b"document", ns_strict)]
    struct Document {
        href: String,
    }

    let doc = Document::parse_str(
        r#"
        <document>
            <href>/asd</href>
        </document>
    "#,
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            href: "/asd".to_owned()
        }
    );
}
