use super::{Variant, attrs::EnumAttrs};
use crate::attrs::VariantAttrs;
use core::panic;
use darling::{FromDeriveInput, FromVariant};
use quote::quote;
use syn::{DataEnum, DeriveInput};

mod impl_de;
mod impl_prop_name;
mod impl_se;

pub struct Enum {
    attrs: EnumAttrs,
    variants: Vec<Variant>,
    ident: syn::Ident,
    generics: syn::Generics,
}

impl Enum {
    pub fn parse(input: &DeriveInput, data: &DataEnum) -> Self {
        let attrs = EnumAttrs::from_derive_input(input).unwrap();

        Self {
            variants: data
                .variants
                .iter()
                .map(|variant| Variant {
                    attrs: VariantAttrs::from_variant(variant).unwrap(),
                    variant: variant.to_owned(),
                })
                .collect(),
            attrs,
            ident: input.ident.to_owned(),
            generics: input.generics.to_owned(),
        }
    }

    pub fn impl_xml_document(&self) -> proc_macro2::TokenStream {
        if self.attrs.untagged.is_present() {
            panic!("XmlDocument only supported for untagged enums");
        }
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;

        quote! {
            impl #impl_generics ::rustical_xml::XmlDocument for #ident #type_generics #where_clause {
                fn parse<R: ::std::io::BufRead>(mut reader: ::quick_xml::NsReader<R>) -> Result<Self, ::rustical_xml::XmlError>
                where
                    Self: ::rustical_xml::XmlDeserialize
                {
                    use ::quick_xml::events::Event;

                    let mut buf = Vec::new();
                    loop {
                        let event = reader.read_event_into(&mut buf)?;
                        let empty = matches!(event, Event::Empty(_));

                        match event {
                            Event::Start(start) | Event::Empty(start) => {
                                return <Self as ::rustical_xml::XmlDeserialize>::deserialize(&mut reader, &start, empty);
                            }
                            Event::Eof => return Err(::rustical_xml::XmlError::Eof),
                            Event::Text(bytes_text) => {
                                return Err(::rustical_xml::XmlError::UnsupportedEvent("Text"));
                            }
                            Event::CData(cdata) => {
                                return Err(::rustical_xml::XmlError::UnsupportedEvent("CDATA"));
                            }
                            Event::Decl(_) => { /* <?xml ... ?> ignore this */ }
                            Event::Comment(_) => { /* ignore */ }
                            Event::DocType(_) => { /* ignore */ }
                            Event::PI(_) => {
                                return Err(::rustical_xml::XmlError::UnsupportedEvent("Processing instruction"));
                            }
                            Event::End(end) => {
                                unreachable!("Premature end of xml document, should be handled by quick_xml");
                            }
                        };
                    }
                }
            }
        }
    }

    pub fn impl_enum_variants(&self) -> proc_macro2::TokenStream {
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;

        if self.attrs.untagged.is_present() {
            let untagged_variants = self.variants.iter().map(|variant| {
                let ty = &variant.deserializer_type();
                quote! { #ty::variant_names() }
            });
            quote! {
                impl #impl_generics ::rustical_xml::EnumVariants for #ident #type_generics #where_clause {
                    const TAGGED_VARIANTS: &'static [(Option<::quick_xml::name::Namespace<'static>>, &'static str)] = &[];

                    fn variant_names() -> Vec<(Option<::quick_xml::name::Namespace<'static>>, &'static str)> {
                        [
                            #(#untagged_variants),*
                        ].concat()
                    }
                }
            }
        } else {
            let tagged_variants = self.variants.iter().map(|variant| {
                let ns = match &variant.attrs.common.ns {
                    Some(ns) => quote! { Some(#ns) },
                    None => quote! { None },
                };
                let b_xml_name = variant.xml_name().value();
                let xml_name = String::from_utf8_lossy(&b_xml_name);
                quote! {(#ns, #xml_name)}
            });

            quote! {
                impl #impl_generics ::rustical_xml::EnumVariants for #ident #type_generics #where_clause {
                    const TAGGED_VARIANTS: &'static [(Option<::quick_xml::name::Namespace<'static>>, &'static str)] = &[
                        #(#tagged_variants),*
                    ];

                    fn variant_names() -> Vec<(Option<::quick_xml::name::Namespace<'static>>, &'static str)> {
                        [Self::TAGGED_VARIANTS,].concat()
                    }
                }
            }
        }
    }
}
