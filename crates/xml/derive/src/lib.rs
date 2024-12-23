use core::panic;
use syn::{parse_macro_input, DeriveInput};

mod de;
use de::{Enum, NamedStruct};

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
