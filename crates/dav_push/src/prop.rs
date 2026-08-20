use rustical_dav::header::Depth;
use rustical_xml::{Unparsed, XmlDeserialize, XmlSerialize};

use crate::VapidPublicKeyB64;

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub struct WebPushTransport {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    pub vapid_public_key: Option<VapidPublicKeyB64>,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub enum Transport {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    WebPush(WebPushTransport),
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub struct Transports {
    #[xml(flatten, ty = "untagged")]
    #[xml(ns = "crate::namespace::NS_DAVPUSH")]
    transports: Vec<Transport>,
}

impl Transports {
    #[must_use]
    pub fn new(vapid_public_key: Option<VapidPublicKeyB64>) -> Self {
        Self {
            transports: vec![Transport::WebPush(WebPushTransport { vapid_public_key })],
        }
    }
}

#[derive(XmlSerialize, XmlDeserialize, PartialEq, Eq, Clone, Debug)]
pub struct SupportedTriggers(#[xml(flatten, ty = "untagged")] pub Vec<Trigger>);

#[derive(XmlSerialize, XmlDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum Trigger {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    ContentUpdate(ContentUpdate),
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    PropertyUpdate(PropertyUpdate),
}

#[derive(XmlSerialize, XmlDeserialize, PartialEq, Eq, Clone, Debug)]
pub struct ContentUpdate(
    #[xml(rename = "depth", ns = "rustical_dav::namespace::NS_DAV")] pub Depth,
);

#[derive(XmlSerialize, PartialEq, Eq, Clone, Debug)]
pub struct PropertyUpdate(
    #[xml(rename = "depth", ns = "rustical_dav::namespace::NS_DAV")] pub Depth,
);

impl XmlDeserialize for PropertyUpdate {
    fn deserialize<R: std::io::BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &quick_xml::events::BytesStart,
        empty: bool,
    ) -> Result<Self, rustical_xml::XmlError> {
        #[derive(XmlDeserialize, PartialEq, Clone, Debug)]
        struct FakePropertyUpdate(
            #[xml(rename = "depth", ns = "rustical_dav::namespace::NS_DAV")] pub Depth,
            #[xml(rename = "prop", ns = "rustical_dav::namespace::NS_DAV")] pub Unparsed,
        );
        let FakePropertyUpdate(depth, _) = FakePropertyUpdate::deserialize(reader, start, empty)?;
        Ok(Self(depth))
    }
}

#[cfg(test)]
mod tests {
    use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

    use crate::{Transports, VapidPublicKeyB64};

    #[derive(XmlRootTag, XmlSerialize)]
    #[xml(root = "document")]
    struct Document {
        transports: Transports,
    }

    #[test]
    fn test_serialize_transports() {
        let pubkey = VapidPublicKeyB64(crate::vapid::tests::PUBLIC_KEY_B64.to_owned());
        let doc = Document {
            transports: Transports::new(Some(pubkey)),
        };
        let xml = doc.serialize_to_string().unwrap();
        insta::assert_snapshot!(xml, @r#"
        <?xml version="1.0" encoding="utf-8"?>
        <document>
            <transports>
                <web-push xmlns="https://bitfire.at/webdav-push">
                    <vapid-public-key xmlns="https://bitfire.at/webdav-push" type="p256ecdsa">BFEnoRsQ3AGqqM3q_7aPGxqVG-oQpSvegEtxK6EppOHlSsUT2RBTFaeZ-3TIvfnJGYcdKlLIPcDimpSLSxq28ik</vapid-public-key>
                </web-push>
            </transports>
        </document>
        "#);
    }

    #[test]
    fn test_serialize_transports_empty() {
        let doc = Document {
            transports: Transports::new(None),
        };
        let xml = doc.serialize_to_string().unwrap();
        insta::assert_snapshot!(xml, @r#"
        <?xml version="1.0" encoding="utf-8"?>
        <document>
            <transports>
                <web-push xmlns="https://bitfire.at/webdav-push">
                </web-push>
            </transports>
        </document>
        "#);
    }
}
