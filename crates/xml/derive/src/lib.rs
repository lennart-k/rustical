use core::panic;
use syn::{parse_macro_input, DeriveInput};

mod de;

use de::{impl_de_enum, impl_de_struct};

#[proc_macro_derive(XmlDeserialize, attributes(xml))]
pub fn derive_xml_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Enum(e) => impl_de_enum(&input, e),
        syn::Data::Struct(s) => impl_de_struct(&input, s),
        syn::Data::Union(_) => panic!("Union not supported"),
    }
    .into()
}
