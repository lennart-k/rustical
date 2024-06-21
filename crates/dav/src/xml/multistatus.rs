use actix_web::{
    body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder, ResponseError,
};
use log::debug;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename = "multistatus")]
pub struct MultistatusElement<T1: Serialize, T2: Serialize> {
    #[serde(rename = "response")]
    pub responses: Vec<T1>,
    #[serde(rename = "response")]
    pub member_responses: Vec<T2>,
    #[serde(rename = "@xmlns")]
    pub ns_dav: &'static str,
    #[serde(rename = "@xmlns:C")]
    pub ns_caldav: &'static str,
    #[serde(rename = "@xmlns:IC")]
    pub ns_ical: &'static str,
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
        debug!("Return multistatus:\n{output}");

        HttpResponse::MultiStatus()
            .content_type(ContentType::xml())
            .body(output)
    }
}
