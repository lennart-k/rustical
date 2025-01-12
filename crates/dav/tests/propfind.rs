use rustical_dav::xml::{PropElement, PropfindElement, PropfindType, Propname};
use rustical_xml::de::XmlDocument;

#[test]
fn propfind_allprop() {
    let propfind = PropfindElement::parse_str(
        r#"
        <propfind xmlns="DAV:">
            <allprop />
        </propfind>
    "#,
    )
    .unwrap();
    assert_eq!(
        propfind,
        PropfindElement {
            prop: PropfindType::Allprop
        }
    );
}

#[test]
fn propfind_propname() {
    let propfind = PropfindElement::parse_str(
        r#"
        <propfind xmlns="DAV:">
            <propname />
        </propfind>
    "#,
    )
    .unwrap();
    assert_eq!(
        propfind,
        PropfindElement {
            prop: PropfindType::Propname
        }
    );
}

#[test]
fn propfind_prop() {
    let propfind = PropfindElement::parse_str(
        r#"
        <propfind xmlns="DAV:">
            <prop>
                <displayname />
                <color />
            </prop>
        </propfind>
    "#,
    )
    .unwrap();
    assert_eq!(
        propfind,
        PropfindElement {
            prop: PropfindType::Prop(PropElement(vec![
                Propname("displayname".to_owned()),
                Propname("color".to_owned()),
            ]))
        }
    );
}

/// Example taken from DAVx5
#[test]
fn propfind_decl() {
    let propfind = PropfindElement::parse_str(
        r#"
        <?xml version='1.0' encoding='UTF-8' ?>
        <propfind xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
            <prop>
                <CARD:max-resource-size />
                <CARD:supported-address-data />
                <supported-report-set />
                <n0:getctag xmlns:n0="http://calendarserver.org/ns/" />
                <sync-token />
            </prop>
        </propfind>
        "#
    ).unwrap();
}
