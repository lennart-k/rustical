use crate::Variant;

use super::Enum;
use quote::quote;

impl Enum {
    fn impl_de_untagged(&self) -> proc_macro2::TokenStream {
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let name = &self.ident;

        let variant_branches = self
            .variants
            .iter()
            .filter_map(|variant| variant.untagged_branch());

        quote! {
            impl #impl_generics ::rustical_xml::XmlDeserialize for #name #type_generics #where_clause {
                fn deserialize<R: ::std::io::BufRead>(
                    reader: &mut ::quick_xml::NsReader<R>,
                    start: &::quick_xml::events::BytesStart,
                    empty: bool
                ) -> Result<Self, rustical_xml::XmlError> {
                    #(#variant_branches);*

                    Err(rustical_xml::XmlError::InvalidVariant("could not match".to_owned()))
                }
            }
        }
    }

    fn impl_de_tagged(&self) -> proc_macro2::TokenStream {
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let name = &self.ident;

        let variant_branches = self.variants.iter().filter_map(Variant::tagged_branch);

        quote! {
            impl #impl_generics ::rustical_xml::XmlDeserialize for #name #type_generics #where_clause {
                fn deserialize<R: std::io::BufRead>(
                    reader: &mut ::quick_xml::NsReader<R>,
                    start: &::quick_xml::events::BytesStart,
                    empty: bool
                ) -> Result<Self, rustical_xml::XmlError> {
                    let (_ns, name) = reader.resolve_element(start.name());

                    match name.as_ref() {
                        #(#variant_branches),*
                        name => {
                            // Handle invalid variant name
                            Err(rustical_xml::XmlError::InvalidVariant(String::from_utf8_lossy(name).to_string()))
                        }
                    }
                }
            }
        }
    }

    pub fn impl_de(&self) -> proc_macro2::TokenStream {
        match self.attrs.untagged.is_present() {
            true => self.impl_de_untagged(),
            false => self.impl_de_tagged(),
        }
    }
}
