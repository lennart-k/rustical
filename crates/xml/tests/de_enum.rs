use rustical_xml::XmlRoot;
use std::io::BufRead;
use xml_derive::XmlDeserialize;

#[test]
fn test_struct_untagged_enum() {
    #[derive(Debug, XmlDeserialize, PartialEq)]
    #[xml(root = b"propfind")]
    struct Propfind {
        prop: Prop,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    struct Prop {
        #[xml(ty = "untagged")]
        prop: PropEnum,
    }

    #[derive(Debug, XmlDeserialize, PartialEq)]
    enum PropEnum {
        A,
        B,
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
