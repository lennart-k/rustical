use std::collections::HashSet;

use rustical_xml::XmlRoot;
use xml_derive::XmlDeserialize;

#[test]
fn test_struct_text_field() {
    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Document {
        #[xml(text)]
        text: String,
        #[xml(text)]
        text2: String,
    }

    impl XmlRoot for Document {
        fn root_tag() -> &'static [u8] {
            b"document"
        }
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
    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Document {
        child: Child,
    }

    #[derive(Debug, XmlDeserialize, PartialEq, Default)]
    struct Child {
        #[xml(text)]
        text: String,
    }

    impl XmlRoot for Document {
        fn root_tag() -> &'static [u8] {
            b"document"
        }
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
    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Document {
        #[xml(rename = "ok-wow")]
        child: Child,
    }

    #[derive(Debug, XmlDeserialize, PartialEq, Default)]
    struct Child {
        #[xml(text)]
        text: String,
    }

    impl XmlRoot for Document {
        fn root_tag() -> &'static [u8] {
            b"document"
        }
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
    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Document {
        #[xml(default = "Default::default")]
        child: Option<Child>,
    }

    impl XmlRoot for Document {
        fn root_tag() -> &'static [u8] {
            b"document"
        }
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
    #[derive(Debug, XmlDeserialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(rename = "child", flatten)]
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
    #[derive(Debug, XmlDeserialize, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        #[xml(rename = "child", flatten)]
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
