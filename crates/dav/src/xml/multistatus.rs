use crate::{namespace::Namespace, xml::TagList};
use actix_web::{
    body::BoxBody,
    http::{header::ContentType, StatusCode},
    HttpRequest, HttpResponse, Responder, ResponseError,
};
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

// Intermediate struct because of a serde limitation, see following article:
// https://stackoverflow.com/questions/78444158/unsupportedcannot-serialize-enum-newtype-variant-exampledata
#[derive(XmlSerialize)]
pub struct PropTagWrapper<T: XmlSerialize> {
    #[xml(flatten, ty = "untagged")]
    pub prop: Vec<T>,
}

// RFC 2518
// <!ELEMENT propstat (prop, status, responsedescription?) >
#[derive(XmlSerialize)]
pub struct PropstatElement<PropType: XmlSerialize> {
    pub prop: PropType,
    #[xml(serialize_with = "xml_serialize_status")]
    pub status: StatusCode,
}

fn xml_serialize_status<W: ::std::io::Write>(
    status: &StatusCode,
    ns: Option<&[u8]>,
    tag: Option<&[u8]>,
    writer: &mut quick_xml::Writer<W>,
) -> std::io::Result<()> {
    XmlSerialize::serialize(&format!("HTTP/1.1 {}", status), ns, tag, writer)
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
pub struct ResponseElement<PropstatType: XmlSerialize> {
    pub href: String,
    #[xml(serialize_with = "xml_serialize_optional_status")]
    pub status: Option<StatusCode>,
    #[xml(flatten)]
    pub propstat: Vec<PropstatWrapper<PropstatType>>,
}

fn xml_serialize_optional_status<W: ::std::io::Write>(
    val: &Option<StatusCode>,
    ns: Option<&[u8]>,
    tag: Option<&[u8]>,
    writer: &mut quick_xml::Writer<W>,
) -> std::io::Result<()> {
    XmlSerialize::serialize(
        &val.map(|status| format!("HTTP/1.1 {}", status)),
        ns,
        tag,
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
#[xml(root = b"multistatus")]
pub struct MultistatusElement<PropType: XmlSerialize, MemberPropType: XmlSerialize> {
    #[xml(rename = b"response", flatten)]
    pub responses: Vec<ResponseElement<PropType>>,
    #[xml(rename = b"response", flatten)]
    pub member_responses: Vec<ResponseElement<MemberPropType>>,
    // TODO: napespaces
    pub ns_dav: &'static str,
    pub ns_davpush: &'static str,
    pub ns_caldav: &'static str,
    pub ns_ical: &'static str,
    pub ns_calendarserver: &'static str,
    pub ns_carddav: &'static str,
    pub sync_token: Option<String>,
}

impl<T1: XmlSerialize, T2: XmlSerialize> Default for MultistatusElement<T1, T2> {
    fn default() -> Self {
        Self {
            responses: vec![],
            member_responses: vec![],
            ns_dav: Namespace::Dav.as_str(),
            ns_davpush: Namespace::DavPush.as_str(),
            ns_caldav: Namespace::CalDAV.as_str(),
            ns_ical: Namespace::ICal.as_str(),
            ns_calendarserver: Namespace::CServer.as_str(),
            ns_carddav: Namespace::CardDAV.as_str(),
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
