use heck::ToKebabCase;
use quote::quote;
use syn::{Fields, FieldsUnnamed};

use super::{attrs::VariantAttrs, get_generic_type};

pub struct Variant {
    pub variant: syn::Variant,
    pub attrs: VariantAttrs,
}

impl Variant {
    fn ident(&self) -> &syn::Ident {
        &self.variant.ident
    }

    pub fn xml_name(&self) -> syn::LitStr {
        self.attrs
            .common
            .rename
            .to_owned()
            .unwrap_or(syn::LitStr::new(
                self.ident().to_string().to_kebab_case().as_str(),
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
            Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => {
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
        let b_variant_name =
            syn::LitByteStr::new(self.xml_name().value().as_bytes(), variant_name.span());
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
                        #b_variant_name => {
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
                        #b_variant_name => {
                            let val = <#deserializer_type as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty)?;
                            Ok(Self::#ident(val))
                        }
                    }
                }
                (false, Fields::Unit, _) => {
                    quote! {
                        #b_variant_name => {
                            // Make sure that content is still consumed
                            <() as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty)?;
                            Ok(Self::#ident)
                        }
                    }
                }
                (true, Fields::Unnamed(FieldsUnnamed { unnamed, .. }), _) => {
                    if unnamed.len() != 1 {
                        panic!("tuple variants should contain exactly one element");
                    }
                    quote! {
                        _ => {
                            let val = <#deserializer_type as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty)?;
                            Ok(Self::#ident(val))
                        }
                    }
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
                    match <#field as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty) {
                        Ok(val) => { return Ok(Self::#ident(val)) }
                        Err(::rustical_xml::XmlError::InvalidVariant(..)) => {}
                        Err(err) => { return Err(err) }
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    // Make sure that content is still consumed
                    match <() as ::rustical_xml::XmlDeserialize>::deserialize(reader, start, empty) {
                        Ok(val) => { return Ok(Self::#ident(val)) }
                        Err(::rustical_xml::XmlError::InvalidVariant(..)) => {}
                        Err(err) => { return Err(err) }
                    }
                }
            }
        })
    }

    pub fn se_branch(&self) -> proc_macro2::TokenStream {
        let ident = self.ident();
        let variant_name = self.xml_name();
        let ns = match &self.attrs.common.ns {
            Some(ns) => quote! { Some(#ns) },
            None => quote! { None },
        };

        match &self.variant.fields {
            Fields::Named(_) => {
                panic!(
                    "struct variants are not supported, please use a tuple variant with a struct"
                )
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                if unnamed.len() != 1 {
                    panic!("tuple variants should contain exactly one element");
                }
                quote! {
                    if let Self::#ident(val) = &self {
                        if !enum_untagged {
                            ::rustical_xml::XmlSerialize::serialize(val, #ns, Some(#variant_name), namespaces, writer)?;
                        } else {
                            ::rustical_xml::XmlSerialize::serialize(val, None, None, namespaces, writer)?;
                        };
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    if let Self::#ident = &self {
                        if !enum_untagged {
                            ::rustical_xml::XmlSerialize::serialize(&(), #ns, Some(#variant_name), namespaces, writer)?;
                        } else {
                            ::rustical_xml::XmlSerialize::serialize(&(), None, None, namespaces, writer)?;
                        };
                    }
                }
            }
        }
    }
}
