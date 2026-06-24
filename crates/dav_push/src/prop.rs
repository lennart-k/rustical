use rustical_dav::header::Depth;
use rustical_xml::{Unparsed, XmlDeserialize, XmlSerialize};

/// The server's VAPID public key, advertised inside `<web-push>` so clients can
/// pin their push subscription to this server (`applicationServerKey`). Per the
/// WebDAV-Push spec: `<vapid-public-key type="p256ecdsa">base64url</…>`.
#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub struct VapidPublicKey {
    #[xml(ty = "attr", rename = "type")]
    pub ty: String,
    #[xml(ty = "text")]
    pub key: String,
}

#[derive(Debug, Clone, Default, XmlSerialize, PartialEq, Eq)]
pub struct WebPushTransport {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    pub vapid_public_key: Option<VapidPublicKey>,
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
    /// Advertise the WebPush transport, including the server's VAPID public key
    /// (base64url) when one is available.
    #[must_use]
    pub fn new(vapid_public_key: Option<String>) -> Self {
        Self {
            transports: vec![Transport::WebPush(WebPushTransport {
                vapid_public_key: vapid_public_key.map(|key| VapidPublicKey {
                    ty: "p256ecdsa".to_owned(),
                    key,
                }),
            })],
        }
    }
}

impl Default for Transports {
    fn default() -> Self {
        Self::new(None)
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
    use super::Transports;
    use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

    #[derive(XmlSerialize, XmlRootTag)]
    #[xml(root = "document")]
    struct Document {
        transports: Transports,
    }

    fn serialize(transports: Transports) -> String {
        Document { transports }.serialize_to_string().unwrap()
    }

    #[test]
    fn advertises_vapid_public_key_when_present() {
        let out = serialize(Transports::new(Some("BNcRexamplekey".to_owned())));
        assert!(out.contains("web-push"), "{out}");
        assert!(out.contains("vapid-public-key"), "{out}");
        assert!(out.contains(r#"type="p256ecdsa""#), "{out}");
        assert!(out.contains("BNcRexamplekey"), "{out}");
    }

    #[test]
    fn web_push_has_no_vapid_key_without_one() {
        let out = serialize(Transports::new(None));
        assert!(out.contains("web-push"), "{out}");
        assert!(!out.contains("vapid-public-key"), "{out}");
    }
}
