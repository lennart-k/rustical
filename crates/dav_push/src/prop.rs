use rustical_dav::header::Depth;
use rustical_xml::{Unparsed, XmlDeserialize, XmlSerialize};

// #[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
// pub struct VapidPublicKey {
//     #[xml(ty = "attr", rename = "type")]
//     pub ty: &'static str,
//     #[xml(ty = "text")]
//     pub key: String,
// }

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub struct WebPushTransport {
    // #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    // pub vapid_public_key: Option<VapidPublicKey>,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub enum Transport {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    WebPush(WebPushTransport),
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub struct Transports(
    #[xml(flatten, ty = "untagged")]
    #[xml(ns = "crate::namespace::NS_DAVPUSH", rename = "transports")]
    pub Vec<Transport>,
);

impl Transports {
    #[must_use]
    pub fn from_p256ecdsa_key(public_key: Option<String>) -> Self {
        Self(vec![Transport::WebPush(WebPushTransport {
            // vapid_public_key: public_key.map(|key| VapidPublicKey {
            //     ty: "p256ecdsa",
            //     key,
            // }),
        })])
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
