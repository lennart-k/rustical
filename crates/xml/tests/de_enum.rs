use rustical_xml::{de::XmlRootParseStr, XmlDeserialize, XmlRoot};
use std::io::BufRead;

#[test]
fn test_struct_untagged_enum() {
    #[derive(Debug, XmlDeserialize, XmlRoot, PartialEq)]
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
                <b/>
            </prop>
        </propfind>
    "#,
    )
    .unwrap();
    assert_eq!(
        doc,
        Propfind {
            prop: Prop { prop: PropEnum::B }
        }
    );
}
