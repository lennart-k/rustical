use quick_xml::events::{BytesStart, Event};
use rustical_xml::{Unit, XmlDeserialize, XmlError, XmlRoot};

#[derive(Debug, XmlDeserialize)]
#[xml(rename_all = "kebab-case")]
pub enum Prop {
    #[xml(rename = "displayname")]
    Displayname(String),
    #[xml(ns = "DAV:Push", rename = "transports")]
    Transports,
}

#[derive(Debug)]
pub struct PropfindElement<T: XmlDeserialize> {
    // child with name propfind and namespace DAV:
    pub prop: Vec<T>,
    pub test: Option<Prop>,
}

impl<T: XmlDeserialize> XmlDeserialize for PropfindElement<T> {
    fn deserialize<R: std::io::BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlError> {
        // init values for the struct attributes
        let mut attr_prop: Option<Vec<T>> = None;
        let mut attr_test: Option<Prop> = None;

        if !empty {
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf)? {
                    Event::End(e) if e.name() == start.name() => {
                        break;
                    }
                    Event::Eof => return Err(XmlError::Eof),
                    Event::Start(start) => {
                        let (_ns, name) = reader.resolve_element(start.name());
                        match name.as_ref() {
                            b"prop" => {
                                if attr_prop.is_none() {
                                    attr_prop = Some(Vec::<T>::deserialize(reader, &start, false)?);
                                }
                            }
                            b"test" => {
                                if attr_test.is_none() {
                                    attr_test = Some(Prop::deserialize(reader, &start, false)?);
                                }
                            }
                            _ => {
                                return Err(XmlError::InvalidTag(
                                    String::from_utf8_lossy(name.as_ref()).to_string(),
                                    "prop".to_string(),
                                ));
                            }
                        }
                    }
                    Event::Empty(start) => {
                        let (_ns, name) = reader.resolve_element(start.name());
                        match name.as_ref() {
                            b"prop" => {
                                if attr_prop.is_none() {
                                    attr_prop = Some(Vec::<T>::deserialize(reader, &start, true)?);
                                }
                            }
                            b"test" => {
                                if attr_test.is_none() {
                                    attr_test = Some(Prop::deserialize(reader, &start, true)?);
                                }
                            }
                            _ => {
                                return Err(XmlError::InvalidTag(
                                    String::from_utf8_lossy(name.as_ref()).to_string(),
                                    "prop".to_string(),
                                ));
                            }
                        }
                    }
                    a => {
                        dbg!(a);
                    }
                };
            }
        }

        let attr_prop = attr_prop.ok_or(XmlError::MissingField("prop"))?;
        Ok(Self {
            prop: attr_prop,
            test: None,
        })
    }
}

impl<T: XmlDeserialize> XmlRoot for PropfindElement<T> {
    fn root_tag() -> &'static [u8] {
        b"propfind"
    }
}

#[test]
fn test_propfind() {
    let propfind: PropfindElement<Prop> = PropfindElement::parse_str(
        r#"
        <propfind xmlns="DAV:" xmlns:P="DAV:Push">
            <P:prop>
                <displayname>Hello!</displayname>
                <transports xmlns="DAV:Push" />
                <transports xmlns="DAV:Push"></transports>
            </P:prop>
            <test>
                <displayname>Okay wow!</displayname>
            </test>
        </propfind>
    "#,
    )
    .unwrap();
    dbg!(propfind);
}

fn asd() {
    let a: Option<String> = None;
}
