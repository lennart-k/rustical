use rustical_xml::{de::XmlDocument, XmlDeserialize, XmlRootTag};

#[test]
fn test_struct_tagged_enum() {
    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"propfind")]
    struct Propfind {
        prop: Prop,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Prop {
        #[xml(ty = "untagged", flatten)]
        prop: Vec<PropEnum>,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Displayname {
        #[xml(ty = "text")]
        name: String,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    enum PropEnum {
        A,
        B,
        Displayname(Displayname),
    }

    let doc = Propfind::parse_str(
        r#"
        <propfind>
            <prop>
                <b/>
                <a/>
                <displayname>Hello!</displayname>
            </prop>
        </propfind>
    "#,
    )
    .unwrap();
    assert_eq!(
        doc,
        Propfind {
            prop: Prop {
                prop: vec![
                    PropEnum::B,
                    PropEnum::A,
                    PropEnum::Displayname(Displayname {
                        name: "Hello!".to_owned()
                    })
                ]
            }
        }
    );
}

#[test]
fn test_tagged_enum_complex() {
    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"propfind")]
    struct Propfind {
        prop: PropStruct,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct PropStruct {
        #[xml(ty = "untagged", flatten)]
        prop: Vec<Prop>,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    enum Prop {
        Nice(Nice),
        #[xml(other)]
        Invalid,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Nice {
        nice: (),
    }

    let _ = Propfind::parse_str(
        r#"
        <propfind>
            <prop>
                <nice>
                    <nice />
                </nice>
                <wtf />
            </prop>
        </propfind>
    "#,
    )
    .unwrap();
}

#[test]
fn test_enum_document() {
    #[derive(Debug, XmlDeserialize, XmlDocument, PartialEq)]
    enum Document {
        Hello,
        Goodbye,
    }
    assert_eq!(
        Document::parse_str(r"<hello></hello>").unwrap(),
        Document::Hello
    );
    assert_eq!(
        Document::parse_str(r"<goodbye/>").unwrap(),
        Document::Goodbye
    );
}

#[test]
fn test_untagged_enum() {
    #[derive(Debug, XmlDeserialize, XmlRootTag, PartialEq)]
    #[xml(root = b"document")]
    struct Document {
        prop: PropElement,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct PropElement {
        #[xml(ty = "untagged", flatten)]
        props: Vec<Prop>,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    #[xml(untagged)]
    enum Prop {
        DefaultProp(DefaultProp),
        ExtensionProp(ExtensionProp),
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    enum DefaultProp {
        Displayname(String),
    }
    #[derive(Debug, XmlDeserialize, PartialEq)]
    enum ExtensionProp {
        Extension(Extension),
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Extension {
        okay: Option<()>,
        nice: Option<()>,
    }

    let doc = Document::parse_str(
        r#"<document>
        <prop>
            <displayname>Hello</displayname>
            <extension><okay /></extension>
            <displayname>Hello</displayname>
        </prop>
    </document>"#,
    )
    .unwrap();
    dbg!(doc);
}
