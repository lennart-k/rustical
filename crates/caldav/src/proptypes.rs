use std::io::Write;

use quick_xml::{events::BytesText, Writer};

pub fn write_string_prop<'a, W: Write>(
    writer: &'a mut Writer<W>,
    propname: &'a str,
    value: &str,
) -> Result<&'a mut Writer<W>, quick_xml::Error> {
    let el = writer.create_element(propname);
    if value.is_empty() {
        el.write_empty()
    } else {
        el.write_text_content(BytesText::new(value))
    }
}

pub fn write_href_prop<'a, W: Write>(
    writer: &'a mut Writer<W>,
    propname: &'a str,
    href: &str,
) -> Result<&'a mut Writer<W>, quick_xml::Error> {
    write_hrefs_prop(writer, propname, vec![href])
}

pub fn write_hrefs_prop<'a, W: Write>(
    writer: &'a mut Writer<W>,
    propname: &'a str,
    hrefs: Vec<&str>,
) -> Result<&'a mut Writer<W>, quick_xml::Error> {
    writer
        .create_element(propname)
        .write_inner_content(|writer| {
            for href in hrefs {
                write_string_prop(writer, "href", href)?;
            }
            Ok(())
        })
}
