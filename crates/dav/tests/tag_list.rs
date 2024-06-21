use rustical_dav::xml::TagList;
use serde::{Deserialize, Serialize};

const INPUT: &str = r#"<Document>
    <prop>
        <nicename/>
        <anotherprop/>
    </prop>
</Document>"#;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct PropElement {
    #[serde(flatten)]
    tags: TagList,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct Document {
    prop: PropElement,
}

fn expected_output() -> Document {
    Document {
        prop: PropElement {
            tags: vec!["nicename".to_owned(), "anotherprop".to_owned()].into(),
        },
    }
}

#[test]
fn test_tagname_deserialize() {
    let result: Document = quick_xml::de::from_str(INPUT).unwrap();
    assert_eq!(result, expected_output());
}

#[test]
fn test_tagname_serialize() {
    let mut result = String::new();
    let mut ser = quick_xml::se::Serializer::new(&mut result);
    ser.indent(' ', 4);

    let to_serialize = &expected_output();
    to_serialize.serialize(ser).unwrap();

    assert_eq!(result, INPUT);
}
