use crate::{
    calendar::generate_propfind_calendar_response,
    depth_extractor::Depth,
    namespace::Namespace,
    propfind::{
        generate_multistatus, parse_propfind, write_invalid_props_response,
        write_propstat_response, write_resourcetype,
    },
    CalDavContext, Error,
};
use actix_web::{
    http::{header::ContentType, StatusCode},
    web::Data,
    HttpRequest, HttpResponse,
};
use actix_web_httpauth::extractors::basic::BasicAuth;
use anyhow::Result;
use quick_xml::{
    events::{BytesText, Event},
    Writer,
};
use rustical_store::calendar::CalendarStore;

// Executes the PROPFIND request and returns a XML string to be written into a <mulstistatus> object.
pub async fn generate_propfind_principal_response(
    props: Vec<&str>,
    principal: &str,
    path: &str,
    prefix: &str,
) -> Result<String, quick_xml::Error> {
    let mut props = props;
    if props.contains(&"allprops") {
        props.extend(
            [
                "resourcetype",
                "current-user-principal",
                "principal-URL",
                "calendar-home-set",
                "calendar-user-address-set",
            ]
            .iter(),
        );
    }

    let mut invalid_props = Vec::<&str>::new();

    let mut output_buffer = Vec::new();
    let mut writer = Writer::new_with_indent(&mut output_buffer, b' ', 2);

    write_propstat_response(&mut writer, path, StatusCode::OK, |writer| {
        let writer = writer;
        for prop in props {
            match prop {
                "resourcetype" => write_resourcetype(writer, vec!["principal", "collection"])?,
                "current-user-principal" | "principal-URL" => {
                    writer.create_element(prop).write_inner_content(|writer| {
                        writer
                            .create_element("href")
                            .write_text_content(BytesText::new(&format!(
                                "{prefix}/{principal}/",
                            )))?;
                        Ok(())
                    })?;
                }
                "calendar-home-set" | "calendar-user-address-set" => {
                    writer
                        .create_element(&format!("C:{prop}"))
                        .write_inner_content(|writer| {
                            writer
                                .create_element("href")
                                .write_text_content(BytesText::new(&format!(
                                    "{prefix}/{principal}/"
                                )))?;
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

pub async fn route_propfind_principal<C: CalendarStore>(
    body: String,
    request: HttpRequest,
    auth: BasicAuth,
    context: Data<CalDavContext<C>>,
    depth: Depth,
) -> Result<HttpResponse, Error> {
    let props = parse_propfind(&body).map_err(|_e| Error::BadRequest)?;

    let mut responses = Vec::new();
    // also get calendars:
    if depth != Depth::Zero {
        let cals = context
            .store
            .read()
            .await
            .get_calendars()
            .await
            .map_err(|_e| Error::InternalError)?;

        for cal in cals {
            responses.push(
                generate_propfind_calendar_response(
                    props.clone(),
                    auth.user_id(),
                    &format!("{}/{}", request.path(), cal.id),
                    &context.prefix,
                    &cal,
                )
                .map_err(|_e| Error::InternalError)?,
            );
        }
    }

    responses.push(
        generate_propfind_principal_response(
            props.clone(),
            auth.user_id(),
            request.path(),
            &context.prefix,
        )
        .await
        .map_err(|_e| Error::InternalError)?,
    );

    let output = generate_multistatus(vec![Namespace::Dav, Namespace::CalDAV], |writer| {
        for response in responses {
            writer.write_event(Event::Text(BytesText::from_escaped(response)))?;
        }
        Ok(())
    })
    .map_err(|_e| Error::InternalError)?;

    println!("{}", &output);

    Ok(HttpResponse::MultiStatus()
        .content_type(ContentType::xml())
        .body(output))
}
