use actix_web::{
    http::{header::ContentType, StatusCode},
    web::Data,
    HttpRequest, HttpResponse,
};
use actix_web_httpauth::extractors::basic::BasicAuth;
use quick_xml::{
    events::{BytesText, Event},
    Writer,
};
use rustical_store::calendar::CalendarStore;

use crate::{
    namespace::Namespace,
    propfind::{
        generate_multistatus, parse_propfind, write_invalid_props_response,
        write_propstat_response, write_resourcetype,
    },
    Context, Error,
};

// Executes the PROPFIND request and returns a XML string to be written into a <mulstistatus> object.
pub async fn generate_propfind_root_response(
    props: Vec<&str>,
    principal: &str,
    path: &str,
    prefix: &str,
) -> Result<String, quick_xml::Error> {
    let mut props = props;
    if props.contains(&"allprops") {
        props.extend(["resourcetype", "current-user-principal"].iter());
    }

    let mut invalid_props = Vec::<&str>::new();

    let mut output_buffer = Vec::new();
    let mut writer = Writer::new_with_indent(&mut output_buffer, b' ', 2);

    write_propstat_response(&mut writer, path, StatusCode::OK, |writer| {
        for prop in props {
            match prop {
                "resourcetype" => write_resourcetype(writer, vec!["collection"])?,
                "current-user-principal" => {
                    writer
                        .create_element("current-user-principal")
                        .write_inner_content(|writer| {
                            writer
                                .create_element("href")
                                .write_text_content(BytesText::new(
                                    // TODO: Replace hard-coded string
                                    &format!("{prefix}/{principal}"),
                                ))?;
                            Ok(())
                        })?;
                }
                "allprops" => {}
                _ => invalid_props.push(prop),
            };
        }
        Ok(())
    })?;
    dbg!(&invalid_props);
    write_invalid_props_response(&mut writer, path, invalid_props)?;
    Ok(std::str::from_utf8(&output_buffer)?.to_string())
}

pub async fn route_propfind_root<C: CalendarStore>(
    body: String,
    request: HttpRequest,
    auth: BasicAuth,
    context: Data<Context<C>>,
) -> Result<HttpResponse, Error> {
    let props = parse_propfind(&body).map_err(|_e| Error::BadRequest)?;

    let responses_string = generate_propfind_root_response(
        props.clone(),
        auth.user_id(),
        request.path(),
        &context.prefix,
    )
    .await
    .map_err(|_e| Error::InternalError)?;

    let output = generate_multistatus(vec![Namespace::Dav, Namespace::CalDAV], |writer| {
        writer.write_event(Event::Text(BytesText::from_escaped(responses_string)))?;
        Ok(())
    })
    .map_err(|_e| Error::InternalError)?;

    Ok(HttpResponse::MultiStatus()
        .content_type(ContentType::xml())
        .body(output))
}
