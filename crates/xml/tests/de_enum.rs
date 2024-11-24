use rustical_xml::XmlRoot;
use xml_derive::XmlDeserialize;

#[test]
fn test_struct_untagged_enum() {
    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Propfind {
        prop: Prop,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Prop {
        #[xml(untagged)]
        prop: PropEnum,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    enum PropEnum {
        A,
        B,
    }

    impl XmlRoot for Propfind {
        fn root_tag() -> &'static [u8] {
            b"propfind"
        }
    }

    let doc = Propfind::parse_str(
        r#"
        <propfind>
            <prop>
                <A/>
            </prop>
        </propfind>
    "#,
    )
    .unwrap();
    assert_eq!(
        doc,
        Propfind {
            prop: Prop { prop: PropEnum::A }
        }
    );
}
