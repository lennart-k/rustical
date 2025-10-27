use crate::xml::TagList;
use headers::{CacheControl, ContentType, HeaderMapExt};
use http::StatusCode;
use quick_xml::name::Namespace;
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};
use std::collections::HashMap;

#[derive(XmlSerialize)]
pub struct PropTagWrapper<T: XmlSerialize>(#[xml(flatten, ty = "untagged")] pub Vec<T>);

// RFC 2518
// <!ELEMENT propstat (prop, status, responsedescription?) >
#[derive(XmlSerialize, Debug)]
pub struct PropstatElement<PropType: XmlSerialize> {
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub prop: PropType,
    #[xml(serialize_with = "xml_serialize_status")]
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub status: StatusCode,
}

fn xml_serialize_status(
    status: &StatusCode,
    ns: Option<Namespace>,
    tag: Option<&str>,
    namespaces: &HashMap<Namespace, &str>,
    writer: &mut quick_xml::Writer<&mut Vec<u8>>,
) -> std::io::Result<()> {
    XmlSerialize::serialize(&format!("HTTP/1.1 {status}"), ns, tag, namespaces, writer)
}

#[derive(XmlSerialize)]
#[xml(untagged)]
pub enum PropstatWrapper<T: XmlSerialize> {
    Normal(PropstatElement<PropTagWrapper<T>>),
    TagList(PropstatElement<TagList>),
}

// RFC 2518
// <!ELEMENT response (href, ((href*, status)|(propstat+)),
// responsedescription?) >
#[derive(XmlSerialize, XmlRootTag)]
#[xml(ns = "crate::namespace::NS_DAV", root = "response")]
#[xml(ns_prefix(
    crate::namespace::NS_DAV = "",
    crate::namespace::NS_CARDDAV = "CARD",
    crate::namespace::NS_CALDAV = "CAL",
    crate::namespace::NS_CALENDARSERVER = "CS",
    crate::namespace::NS_DAVPUSH = "PUSH"
))]
pub struct ResponseElement<PropstatType: XmlSerialize> {
    pub href: String,
    #[xml(serialize_with = "xml_serialize_optional_status")]
    pub status: Option<StatusCode>,
    #[xml(flatten)]
    pub propstat: Vec<PropstatWrapper<PropstatType>>,
}

fn xml_serialize_optional_status(
    val: &Option<StatusCode>,
    ns: Option<Namespace>,
    tag: Option<&str>,
    namespaces: &HashMap<Namespace, &str>,
    writer: &mut quick_xml::Writer<&mut Vec<u8>>,
) -> std::io::Result<()> {
    XmlSerialize::serialize(
        &val.map(|status| format!("HTTP/1.1 {status}")),
        ns,
        tag,
        namespaces,
        writer,
    )
}

impl<PT: XmlSerialize> Default for ResponseElement<PT> {
    fn default() -> Self {
        Self {
            href: String::new(),
            status: None,
            propstat: vec![],
        }
    }
}

// RFC 2518
// <!ELEMENT multistatus (response+, responsedescription?) >
// Extended by sync-token as specified in RFC 6578
#[derive(XmlSerialize, XmlRootTag)]
#[xml(root = "multistatus", ns = "crate::namespace::NS_DAV")]
#[xml(ns_prefix(
    crate::namespace::NS_DAV = "",
    crate::namespace::NS_CARDDAV = "CARD",
    crate::namespace::NS_CALDAV = "CAL",
    crate::namespace::NS_CALENDARSERVER = "CS",
    crate::namespace::NS_DAVPUSH = "PUSH"
))]
pub struct MultistatusElement<PropType: XmlSerialize, MemberPropType: XmlSerialize> {
    #[xml(rename = "response", flatten)]
    pub responses: Vec<ResponseElement<PropType>>,
    #[xml(rename = "response", flatten)]
    pub member_responses: Vec<ResponseElement<MemberPropType>>,
    pub sync_token: Option<String>,
}

impl<T1: XmlSerialize, T2: XmlSerialize> Default for MultistatusElement<T1, T2> {
    fn default() -> Self {
        Self {
            responses: vec![],
            member_responses: vec![],
            sync_token: None,
        }
    }
}

impl<T1: XmlSerialize, T2: XmlSerialize> axum::response::IntoResponse
    for MultistatusElement<T1, T2>
{
    fn into_response(self) -> axum::response::Response {
        use axum::body::Body;

        let output = match self.serialize_to_string() {
            Ok(out) => out,
            Err(err) => return crate::Error::from(err).into_response(),
        };

        let mut resp = axum::response::Response::builder().status(StatusCode::MULTI_STATUS);
        let hdrs = resp.headers_mut().unwrap();
        hdrs.typed_insert(ContentType::xml());
        hdrs.typed_insert(CacheControl::new().with_no_cache());
        resp.body(Body::from(output)).unwrap()
    }
}
