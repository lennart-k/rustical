use crate::{namespace::Namespace, xml::TagList};
use actix_web::{
    body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder, ResponseError,
};
use serde::Serialize;

// Intermediate struct because of a serde limitation, see following article:
// https://stackoverflow.com/questions/78444158/unsupportedcannot-serialize-enum-newtype-variant-exampledata
#[derive(Serialize)]
pub struct PropTagWrapper<T: Serialize> {
    #[serde(rename = "$value")]
    pub prop: Vec<T>,
}

// RFC 2518
// <!ELEMENT propstat (prop, status, responsedescription?) >
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PropstatElement<PropType: Serialize> {
    pub prop: PropType,
    pub status: String,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum PropstatWrapper<T: Serialize> {
    Normal(PropstatElement<PropTagWrapper<T>>),
    TagList(PropstatElement<TagList>),
}

// RFC 2518
// <!ELEMENT response (href, ((href*, status)|(propstat+)),
// responsedescription?) >
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ResponseElement<PropstatType: Serialize> {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub propstat: Vec<PropstatType>,
}

impl<PT: Serialize> Default for ResponseElement<PT> {
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
#[derive(Serialize)]
#[serde(rename = "multistatus", rename_all = "kebab-case")]
pub struct MultistatusElement<T1: Serialize, T2: Serialize> {
    #[serde(rename = "response")]
    pub responses: Vec<ResponseElement<T1>>,
    #[serde(rename = "response")]
    pub member_responses: Vec<ResponseElement<T2>>,
    #[serde(rename = "@xmlns")]
    pub ns_dav: &'static str,
    #[serde(rename = "@xmlns:C")]
    pub ns_caldav: &'static str,
    #[serde(rename = "@xmlns:IC")]
    pub ns_ical: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_token: Option<String>,
}

impl<T1: Serialize, T2: Serialize> Default for MultistatusElement<T1, T2> {
    fn default() -> Self {
        Self {
            responses: vec![],
            member_responses: vec![],
            ns_dav: Namespace::Dav.as_str(),
            ns_caldav: Namespace::CalDAV.as_str(),
            ns_ical: Namespace::ICal.as_str(),
            sync_token: None,
        }
    }
}

impl<T1: Serialize, T2: Serialize> Responder for MultistatusElement<T1, T2> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut output = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n".to_owned();
        let mut ser = quick_xml::se::Serializer::new(&mut output);
        ser.indent(' ', 4);
        if let Err(err) = self.serialize(ser) {
            return crate::Error::from(err).error_response();
        }

        HttpResponse::MultiStatus()
            .content_type(ContentType::xml())
            .body(output)
    }
}
