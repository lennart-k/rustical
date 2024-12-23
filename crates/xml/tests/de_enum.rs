use rustical_xml::{de::XmlRootParseStr, Unit, XmlDeserialize, XmlRoot};

#[test]
fn test_struct_tagged_enum() {
    #[derive(Debug, XmlDeserialize, XmlRoot, PartialEq)]
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
    #[derive(Debug, XmlDeserialize, XmlRoot, PartialEq)]
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
        nice: Unit,
    }

    let asd = Propfind::parse_str(
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
    dbg!(asd);
}
