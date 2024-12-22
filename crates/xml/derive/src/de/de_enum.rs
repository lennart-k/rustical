use crate::de::attrs::VariantAttrs;
use darling::{FromDeriveInput, FromVariant};
use heck::ToKebabCase;
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, FieldsUnnamed, Variant};

use super::attrs::EnumAttrs;

pub struct Enum {
    attrs: EnumAttrs,
    variants: Vec<syn::Variant>,
    ident: syn::Ident,
    generics: syn::Generics,
}

impl Enum {
    pub fn impl_de(&self) -> proc_macro2::TokenStream {
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let name = &self.ident;

        let variants = self.variants.iter().map(|variant| {
            let attrs = VariantAttrs::from_variant(variant).unwrap();
            let variant_name = attrs.common.rename.unwrap_or(syn::LitByteStr::new(
                variant.ident.to_string().to_kebab_case().as_bytes(),
                variant.ident.span(),
            ));
            let branch = enum_variant_branch(variant);

            quote! { #variant_name => { #branch } }
        });

        quote! {
            impl #impl_generics ::rustical_xml::XmlDeserialize for #name #type_generics #where_clause {
                fn deserialize<R: std::io::BufRead>(
                    reader: &mut quick_xml::NsReader<R>,
                    start: &quick_xml::events::BytesStart,
                    empty: bool
                ) -> Result<Self, rustical_xml::XmlDeError> {
                    use quick_xml::events::Event;

                    let (_ns, name) = reader.resolve_element(start.name());

                    match name.as_ref() {
                        #(#variants),*
                        name => {
                            // Handle invalid variant name
                            Err(rustical_xml::XmlDeError::InvalidVariant(String::from_utf8_lossy(name).to_string()))
                        }
                    }
                }
            }
        }
    }

    pub fn parse(input: &DeriveInput, data: &DataEnum) -> Self {
        let attrs = EnumAttrs::from_derive_input(input).unwrap();

        Self {
            attrs,
            variants: data.variants.iter().cloned().collect(),
            ident: input.ident.to_owned(),
            generics: input.generics.to_owned(),
        }
    }
}

pub fn enum_variant_branch(variant: &Variant) -> proc_macro2::TokenStream {
    let ident = &variant.ident;

    match &variant.fields {
        Fields::Named(_) => {
            panic!("struct variants are not supported, please use a tuple variant with a struct")
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            if unnamed.len() != 1 {
                panic!("tuple variants should contain exactly one element");
            }
            let field = unnamed.iter().next().unwrap();
            quote! {
                let val = #field::deserialize(reader, start, empty)?;
                Ok(Self::#ident(val))
            }
        }
        Fields::Unit => {
            quote! {
                // Make sure that content is still consumed
                ::rustical_xml::Unit::deserialize(reader, start, empty)?;
                Ok(Self::#ident)
            }
        }
    }
}
