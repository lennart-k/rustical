use std::collections::HashMap;

use crate::xml::TagList;
use actix_web::{
    body::BoxBody,
    http::{header::ContentType, StatusCode},
    HttpRequest, HttpResponse, Responder, ResponseError,
};
use axum::{http::Response, response::IntoResponse};
use quick_xml::name::Namespace;
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

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

fn xml_serialize_status<W: ::std::io::Write>(
    status: &StatusCode,
    ns: Option<Namespace>,
    tag: Option<&[u8]>,
    namespaces: &HashMap<Namespace, &[u8]>,
    writer: &mut quick_xml::Writer<W>,
) -> std::io::Result<()> {
    XmlSerialize::serialize(&format!("HTTP/1.1 {}", status), ns, tag, namespaces, writer)
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
#[derive(XmlSerialize)]
#[xml(ns = "crate::namespace::NS_DAV")]
pub struct ResponseElement<PropstatType: XmlSerialize> {
    pub href: String,
    #[xml(serialize_with = "xml_serialize_optional_status")]
    pub status: Option<StatusCode>,
    #[xml(flatten)]
    pub propstat: Vec<PropstatWrapper<PropstatType>>,
}

fn xml_serialize_optional_status<W: ::std::io::Write>(
    val: &Option<StatusCode>,
    ns: Option<Namespace>,
    tag: Option<&[u8]>,
    namespaces: &HashMap<Namespace, &[u8]>,
    writer: &mut quick_xml::Writer<W>,
) -> std::io::Result<()> {
    XmlSerialize::serialize(
        &val.map(|status| format!("HTTP/1.1 {}", status)),
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
#[xml(root = b"multistatus", ns = "crate::namespace::NS_DAV")]
#[xml(ns_prefix(
    crate::namespace::NS_DAV = b"",
    crate::namespace::NS_CARDDAV = b"CARD",
    crate::namespace::NS_CALDAV = b"CAL",
    crate::namespace::NS_CALENDARSERVER = b"CS",
    crate::namespace::NS_DAVPUSH = b"PUSH"
))]
pub struct MultistatusElement<PropType: XmlSerialize, MemberPropType: XmlSerialize> {
    #[xml(rename = b"response", flatten)]
    pub responses: Vec<ResponseElement<PropType>>,
    #[xml(rename = b"response", flatten)]
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

impl<T1: XmlSerialize, T2: XmlSerialize> Responder for MultistatusElement<T1, T2> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut output: Vec<_> = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n".into();
        let mut writer = quick_xml::Writer::new_with_indent(&mut output, b' ', 4);
        if let Err(err) = self.serialize_root(&mut writer) {
            return crate::Error::from(err).error_response();
        }

        HttpResponse::MultiStatus()
            .content_type(ContentType::xml())
            .body(String::from_utf8(output).unwrap())
    }
}

impl<T1: XmlSerialize, T2: XmlSerialize> IntoResponse for MultistatusElement<T1, T2> {
    fn into_response(self) -> axum::response::Response {
        let mut output: Vec<_> = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n".into();
        let mut writer = quick_xml::Writer::new_with_indent(&mut output, b' ', 4);
        if let Err(err) = self.serialize_root(&mut writer) {
            return crate::Error::from(err).into_response();
        }
        Response::builder()
            .header("Content-Type", "application/xml")
            .status(axum::http::StatusCode::MULTI_STATUS)
            .body(String::from_utf8(output).unwrap().into())
            .unwrap()
    }
}
