use std::io::Write;

use actix_web::http::StatusCode;
use anyhow::{anyhow, Result};
use quick_xml::{
    events::{attributes::Attribute, BytesText},
    Writer,
};

pub fn parse_propfind(body: &str) -> Result<Vec<&str>> {
    if body.is_empty() {
        // if body is empty, allprops must be returned (RFC 4918)
        return Ok(vec!["allprops"]);
    }
    let doc = roxmltree::Document::parse(body)?;

    let propfind_node = doc.root_element();
    if propfind_node.tag_name().name() != "propfind" {
        return Err(anyhow!("invalid tag"));
    }

    let prop_node = if let Some(el) = propfind_node.first_element_child() {
        el
    } else {
        return Ok(Vec::new());
    };

    let props = match prop_node.tag_name().name() {
        "prop" => Ok(prop_node
            .children()
            .map(|node| node.tag_name().name())
            .collect()),
        _ => Err(anyhow!("invalid prop tag")),
    };
    dbg!(body, &props);
    props
}

pub fn write_resourcetype<W: Write>(
    writer: &mut Writer<W>,
    types: Vec<&str>,
) -> Result<(), quick_xml::Error> {
    writer
        .create_element("resourcetype")
        .write_inner_content(|writer| {
            for resourcetype in types {
                writer.create_element(resourcetype).write_empty()?;
            }
            Ok(())
        })?;
    Ok(())
}

pub fn write_invalid_props_response<W: Write>(
    writer: &mut Writer<W>,
    href: &str,
    invalid_props: Vec<&str>,
) -> Result<(), quick_xml::Error> {
    if invalid_props.is_empty() {
        return Ok(());
    };

    write_propstat_response(writer, href, StatusCode::NOT_FOUND, |writer| {
        for prop in invalid_props {
            writer.create_element(prop).write_empty()?;
        }
        Ok(())
    })?;

    Ok(())
}

pub fn write_propstat_element<F, W: Write>(
    writer: &mut Writer<W>,
    status: StatusCode,
    prop_closure: F,
) -> Result<(), quick_xml::Error>
where
    F: FnOnce(&mut Writer<W>) -> Result<(), quick_xml::Error>,
{
    writer
        .create_element("propstat")
        .write_inner_content(|writer| {
            writer
                .create_element("prop")
                .write_inner_content(prop_closure)?;

            writer
                .create_element("status")
                .write_text_content(BytesText::new(&format!("HTTP/1.1 {}", status)))?;
            Ok(())
        })?;
    Ok(())
}

// Writes a propstat response into a multistatus
// closure hooks into the <prop> element
pub fn write_propstat_response<F, W: Write>(
    writer: &mut Writer<W>,
    href: &str,
    status: StatusCode,
    prop_closure: F,
) -> Result<(), quick_xml::Error>
where
    F: FnOnce(&mut Writer<W>) -> Result<(), quick_xml::Error>,
{
    writer
        .create_element("response")
        .write_inner_content(|writer| {
            writer
                .create_element("href")
                .write_text_content(BytesText::new(href))?;

            write_propstat_element(writer, status, prop_closure)?;

            Ok(())
        })?;
    Ok(())
}

pub fn generate_multistatus<'a, F, A>(namespaces: A, closure: F) -> Result<String>
where
    F: FnOnce(&mut Writer<&mut Vec<u8>>) -> Result<(), quick_xml::Error>,
    A: IntoIterator,
    A::Item: Into<Attribute<'a>>,
{
    let mut output_buffer = Vec::new();
    let mut writer = Writer::new_with_indent(&mut output_buffer, b' ', 2);
    writer
        .create_element("multistatus")
        .with_attributes(namespaces)
        .write_inner_content(closure)?;

    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{}",
        std::str::from_utf8(&output_buffer)?
    ))
}

pub fn generate_mkcol_response<'a, F, A>(namespaces: A, closure: F) -> Result<String>
where
    F: FnOnce(&mut Writer<&mut Vec<u8>>) -> Result<(), quick_xml::Error>,
    A: IntoIterator,
    A::Item: Into<Attribute<'a>>,
{
    let mut output_buffer = Vec::new();
    let mut writer = Writer::new_with_indent(&mut output_buffer, b' ', 2);
    writer
        .create_element("mkcol-response")
        .with_attributes(namespaces)
        .write_inner_content(closure)?;

    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{}",
        std::str::from_utf8(&output_buffer)?
    ))
}
