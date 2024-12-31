use super::{attrs::EnumAttrs, Variant};
use crate::attrs::VariantAttrs;
use core::panic;
use darling::{FromDeriveInput, FromVariant};
use quote::quote;
use syn::{DataEnum, DeriveInput};

pub struct Enum {
    attrs: EnumAttrs,
    variants: Vec<Variant>,
    ident: syn::Ident,
    generics: syn::Generics,
}

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
                    reader: &mut quick_xml::NsReader<R>,
                    start: &quick_xml::events::BytesStart,
                    empty: bool
                ) -> Result<Self, rustical_xml::XmlDeError> {
                    #(#variant_branches);*

                    Err(rustical_xml::XmlDeError::UnknownError)
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
                    reader: &mut quick_xml::NsReader<R>,
                    start: &quick_xml::events::BytesStart,
                    empty: bool
                ) -> Result<Self, rustical_xml::XmlDeError> {
                    let (_ns, name) = reader.resolve_element(start.name());

                    match name.as_ref() {
                        #(#variant_branches),*
                        name => {
                            // Handle invalid variant name
                            Err(rustical_xml::XmlDeError::InvalidVariant(String::from_utf8_lossy(name).to_string()))
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

    pub fn impl_se(&self) -> proc_macro2::TokenStream {
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;
        let enum_untagged = self.attrs.untagged.is_present();
        let variant_serializers = self.variants.iter().map(Variant::se_branch);

        quote! {
            impl #impl_generics ::rustical_xml::XmlSerialize for #ident #type_generics #where_clause {
                fn serialize<W: ::std::io::Write>(
                    &self,
                    ns: Option<::quick_xml::name::Namespace>,
                    tag: Option<&[u8]>,
                    namespaces: &::std::collections::HashMap<::quick_xml::name::Namespace, &[u8]>,
                    writer: &mut ::quick_xml::Writer<W>
                ) -> ::std::io::Result<()> {
                    use ::quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

                    let prefix = ns
                        .map(|ns| namespaces.get(&ns))
                        .unwrap_or(None)
                        .map(|prefix| [*prefix, b":"].concat());
                    let has_prefix = prefix.is_some();
                    let tagname = tag.map(|tag| [&prefix.unwrap_or_default(), tag].concat());
                    let qname = tagname.as_ref().map(|tagname| ::quick_xml::name::QName(tagname));

                    const enum_untagged: bool = #enum_untagged;

                    if let Some(qname) = &qname {
                        let mut bytes_start = BytesStart::from(qname.to_owned());
                        if !has_prefix {
                            if let Some(ns) = &ns {
                                bytes_start.push_attribute((b"xmlns".as_ref(), ns.as_ref()));
                            }
                        }
                        writer.write_event(Event::Start(bytes_start))?;
                    }

                    #(#variant_serializers);*

                    if let Some(qname) = &qname {
                        writer.write_event(Event::End(BytesEnd::from(qname.to_owned())))?;
                    }
                    Ok(())
                }

                fn attributes<'a>(&self) -> Option<Vec<::quick_xml::events::attributes::Attribute<'a>>> {
                    None
                }
            }
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
                fn parse<R: ::std::io::BufRead>(mut reader: ::quick_xml::NsReader<R>) -> Result<Self, ::rustical_xml::XmlDeError>
                where
                    Self: ::rustical_xml::XmlDeserialize
                {
                    use ::quick_xml::events::Event;

                    let mut buf = Vec::new();
                    loop {
                        let event = reader.read_event_into(&mut buf)?;
                        let empty = matches!(event, Event::Empty(_));

                        match event {
                            Event::Decl(_) => { /* <?xml ... ?> ignore this */ }
                            Event::Comment(_) => { /*  ignore this */ }
                            Event::Start(start) | Event::Empty(start) => {
                                return <Self as ::rustical_xml::XmlDeserialize>::deserialize(&mut reader, &start, empty);
                            }
                            _ => return Err(::rustical_xml::XmlDeError::UnknownError),
                        };
                    }
                }
            }
        }
    }
}
