use core::panic;
use syn::{parse_macro_input, DeriveInput};

pub(crate) mod attrs;
mod common;
mod field;
mod variant;
mod xml_enum;
mod xml_struct;

pub(crate) use common::*;
pub(crate) use field::Field;
pub(crate) use variant::Variant;
pub(crate) use xml_enum::Enum;
pub(crate) use xml_struct::NamedStruct;

#[proc_macro_derive(XmlDeserialize, attributes(xml))]
pub fn derive_xml_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Enum(e) => Enum::parse(&input, e).impl_de(),
        syn::Data::Struct(s) => NamedStruct::parse(&input, s).impl_de(),
        syn::Data::Union(_) => panic!("Union not supported"),
    }
    .into()
}

#[proc_macro_derive(XmlSerialize, attributes(xml))]
pub fn derive_xml_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Enum(e) => Enum::parse(&input, e).impl_se(),
        syn::Data::Struct(s) => NamedStruct::parse(&input, s).impl_se(),
        syn::Data::Union(_) => panic!("Union not supported"),
    }
    .into()
}

#[proc_macro_derive(XmlRootTag, attributes(xml))]
pub fn derive_xml_root_tag(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Struct(s) => NamedStruct::parse(&input, s).impl_xml_root_tag(),
        syn::Data::Enum(_) => panic!("Enum not supported as root"),
        syn::Data::Union(_) => panic!("Union not supported as root"),
    }
    .into()
}

#[proc_macro_derive(XmlDocument, attributes(xml))]
pub fn derive_xml_document(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Struct(_) => panic!("Struct not supported, use XmlRootTag instead"),
        syn::Data::Enum(e) => Enum::parse(&input, e).impl_xml_document(),
        syn::Data::Union(_) => panic!("Union not supported as root"),
    }
    .into()
}

#[proc_macro_derive(EnumVariants, attributes(xml))]
pub fn derive_enum_variants(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Struct(_) => panic!("Struct not supported"),
        syn::Data::Enum(e) => Enum::parse(&input, e).impl_enum_variants(),
        syn::Data::Union(_) => panic!("Union not supported"),
    }
    .into()
}

#[proc_macro_derive(EnumUnitVariants, attributes(xml))]
pub fn derive_enum_unit_variants(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Struct(_) => panic!("Struct not supported"),
        syn::Data::Enum(e) => Enum::parse(&input, e).impl_enum_unit_variants(),
        syn::Data::Union(_) => panic!("Union not supported"),
    }
    .into()
}
