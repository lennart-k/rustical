use core::panic;

use super::{attrs::EnumAttrs, get_generic_type};
use crate::de::attrs::VariantAttrs;
use darling::{FromDeriveInput, FromVariant};
use heck::ToKebabCase;
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, FieldsUnnamed};

pub struct Variant {
    variant: syn::Variant,
    attrs: VariantAttrs,
}

impl Variant {
    fn ident(&self) -> &syn::Ident {
        &self.variant.ident
    }

    pub fn xml_name(&self) -> syn::LitByteStr {
        self.attrs
            .common
            .rename
            .to_owned()
            .unwrap_or(syn::LitByteStr::new(
                self.ident().to_string().to_kebab_case().as_bytes(),
                self.ident().span(),
            ))
    }

    fn skip_de(&self) -> bool {
        self.attrs.skip_deserializing.is_present()
    }

    fn variant_type(&self) -> syn::Type {
        match &self.variant.fields {
            Fields::Named(_) => panic!(
                "struct variants are not supported, please use a tuple variant with a struct"
            ),
            Fields::Unit => syn::Type::Path(syn::parse_str("::rustical_xml::Unit").unwrap()),
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                if unnamed.len() != 1 {
                    panic!("tuple variants should contain exactly one element");
                }
                let field = unnamed.iter().next().unwrap();
                field.ty.to_owned()
            }
        }
    }

    fn is_optional(&self) -> bool {
        if let syn::Type::Path(syn::TypePath { path, .. }) = self.variant_type() {
            if path.segments.len() != 1 {
                return false;
            }
            let type_ident = &path.segments.first().unwrap().ident;
            let option: syn::Ident = syn::parse_str("Option").unwrap();
            return type_ident == &option;
        }
        false
    }

    /// The type to deserialize to
    /// - type Option<T> => optional: deserialize with T
    /// - flatten Vec<T>: deserialize with T
    /// - deserialize with T
    pub fn deserializer_type(&self) -> syn::Type {
        let ty = self.variant_type();
        if self.is_optional() {
            return get_generic_type(&ty).unwrap().to_owned();
        }
        ty
    }

    pub fn tagged_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.skip_de() {
            return None;
        }
        let ident = self.ident();
        let variant_name = self.xml_name();
        let deserializer_type = self.deserializer_type();

        Some(
            match (
                self.attrs.other.is_present(),
                &self.variant.fields,
                self.is_optional(),
            ) {
                (_, Fields::Named(_), _) => {
                    panic!(
                    "struct variants are not supported, please use a tuple variant with a struct"
                )
                }
                (false, Fields::Unnamed(FieldsUnnamed { unnamed, .. }), true) => {
                    if unnamed.len() != 1 {
                        panic!("tuple variants should contain exactly one element");
                    }
                    quote! {
                        #variant_name => {
                            let val = Some(<#deserializer_type as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty)?);
                            Ok(Self::#ident(val))
                        }
                    }
                }
                (false, Fields::Unnamed(FieldsUnnamed { unnamed, .. }), false) => {
                    if unnamed.len() != 1 {
                        panic!("tuple variants should contain exactly one element");
                    }
                    quote! {
                        #variant_name => {
                            let val = <#deserializer_type as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty)?;
                            Ok(Self::#ident(val))
                        }
                    }
                }
                (false, Fields::Unit, _) => {
                    quote! {
                        #variant_name => {
                            // Make sure that content is still consumed
                            <() as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty)?;
                            Ok(Self::#ident)
                        }
                    }
                }
                (true, Fields::Unnamed(_), _) => {
                    panic!("other for tuple enums not implemented yet")
                }
                (true, Fields::Unit, _) => {
                    quote! {
                        _ => {
                            // Make sure that content is still consumed
                            <() as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty)?;
                            Ok(Self::#ident)
                        }
                    }
                }
            },
        )
    }

    pub fn untagged_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.skip_de() {
            return None;
        }
        if self.attrs.other.is_present() {
            panic!("using the other flag on an untagged variant is futile");
        }

        let ident = self.ident();

        Some(match &self.variant.fields {
            Fields::Named(_) => {
                panic!(
                    "struct variants are not supported, please use a tuple variant with a struct"
                )
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                if unnamed.len() != 1 {
                    panic!("tuple variants should contain exactly one element");
                }
                let field = unnamed.iter().next().unwrap();
                quote! {
                    if let Ok(val) = <#field as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty) {
                        return Ok(Self::#ident(val));
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    // Make sure that content is still consumed
                    if let Ok(_) = <() as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty) {
                        return Ok(Self::#ident);
                    }
                }
            }
        })
    }
}

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

        // TODO: Implement attributes
        quote! {
            impl #impl_generics ::rustical_xml::XmlSerialize for #ident #type_generics #where_clause {
                fn serialize<W: ::std::io::Write>(
                    &self,
                    tag: Option<&[u8]>,
                    writer: &mut ::quick_xml::Writer<W>
                ) -> ::std::io::Result<()> {
                    use ::quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

                    let tag_str = tag.map(String::from_utf8_lossy);

                    if let Some(tag) = &tag_str {
                        writer.write_event(Event::Start(BytesStart::new(tag.to_owned())))?;
                    }
                    if let Some(tag) = &tag_str {
                        writer.write_event(Event::End(BytesEnd::new(tag.to_owned())))?;
                    }
                    Ok(())
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
