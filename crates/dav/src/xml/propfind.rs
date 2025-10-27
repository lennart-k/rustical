use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use rustical_xml::NamespaceOwned;
use rustical_xml::Unparsed;
use rustical_xml::XmlDeserialize;
use rustical_xml::XmlError;
use rustical_xml::XmlRootTag;

#[derive(Debug, Clone, XmlDeserialize, XmlRootTag, PartialEq)]
#[xml(root = "propfind", ns = "crate::namespace::NS_DAV")]
pub struct PropfindElement<PN: XmlDeserialize> {
    #[xml(ty = "untagged")]
    pub prop: PropfindType<PN>,
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub include: Option<PropElement<PN>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropElement<PN: XmlDeserialize>(
    // valid
    pub Vec<PN>,
    // invalid
    pub Vec<(Option<NamespaceOwned>, String)>,
);

impl<PN: XmlDeserialize> XmlDeserialize for PropElement<PN> {
    fn deserialize<R: std::io::BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &quick_xml::events::BytesStart,
        empty: bool,
    ) -> Result<Self, XmlError> {
        if empty {
            return Ok(Self(vec![], vec![]));
        }
        let mut buf = Vec::new();
        let mut valid_props = vec![];
        let mut invalid_props = vec![];
        loop {
            let event = reader.read_event_into(&mut buf)?;
            match &event {
                Event::End(e) if e.name() == start.name() => {
                    break;
                }
                Event::Eof => return Err(XmlError::Eof),
                // start of a child element
                Event::Start(start) | Event::Empty(start) => {
                    let empty = matches!(event, Event::Empty(_));
                    let (ns, name) = reader.resolve_element(start.name());
                    let ns = match ns {
                        ResolveResult::Bound(ns) => Some(NamespaceOwned::from(ns)),
                        ResolveResult::Unknown(_ns) => todo!("handle error"),
                        ResolveResult::Unbound => None,
                    };

                    match PN::deserialize(reader, start, empty) {
                        Ok(propname) => valid_props.push(propname),
                        Err(XmlError::InvalidVariant(_)) => {
                            invalid_props
                                .push((ns, String::from_utf8_lossy(name.as_ref()).to_string()));
                            // Consume content
                            Unparsed::deserialize(reader, start, empty)?;
                        }
                        Err(err) => return Err(err),
                    }
                }
                Event::Text(_) | Event::CData(_) => {
                    return Err(XmlError::UnsupportedEvent("Not expecting text here"));
                }
                Event::GeneralRef(_) => {
                    return Err(::rustical_xml::XmlError::UnsupportedEvent("GeneralRef"));
                }
                Event::Decl(_) | Event::Comment(_) | Event::DocType(_) | Event::PI(_) => { /* ignore */
                }
                Event::End(_end) => {
                    unreachable!(
                        "Unexpected closing tag for wrong element, should be handled by quick_xml"
                    );
                }
            }
        }
        Ok(Self(valid_props, invalid_props))
    }
}

#[derive(Debug, Clone, XmlDeserialize, PartialEq, Eq)]
pub enum PropfindType<PN: XmlDeserialize> {
    #[xml(ns = "crate::namespace::NS_DAV")]
    Propname,
    #[xml(ns = "crate::namespace::NS_DAV")]
    Allprop,
    #[xml(ns = "crate::namespace::NS_DAV")]
    Prop(PropElement<PN>),
}
