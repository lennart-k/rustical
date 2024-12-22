use super::attrs::EnumAttrs;
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

    pub fn tagged_branch(&self) -> proc_macro2::TokenStream {
        let ident = self.ident();
        let variant_name = self.xml_name();

        match (self.attrs.other.is_present(), &self.variant.fields) {
            (_, Fields::Named(_)) => {
                panic!(
                    "struct variants are not supported, please use a tuple variant with a struct"
                )
            }
            (false, Fields::Unnamed(FieldsUnnamed { unnamed, .. })) => {
                if unnamed.len() != 1 {
                    panic!("tuple variants should contain exactly one element");
                }
                let field = unnamed.iter().next().unwrap();
                quote! {
                    #variant_name => {
                        let val = #field::deserialize(reader, start, empty)?;
                        Ok(Self::#ident(val))
                    }
                }
            }
            (false, Fields::Unit) => {
                quote! {
                    #variant_name => {
                        // Make sure that content is still consumed
                        ::rustical_xml::Unit::deserialize(reader, start, empty)?;
                        Ok(Self::#ident)
                    }
                }
            }
            (true, Fields::Unnamed(_)) => {
                panic!("other for tuple enums not implemented yet")
            }
            (true, Fields::Unit) => {
                quote! {
                    _ => {
                        // Make sure that content is still consumed
                        ::rustical_xml::Unit::deserialize(reader, start, empty)?;
                        Ok(Self::#ident)
                    }
                }
            }
        }
    }

    pub fn untagged_branch(&self) -> proc_macro2::TokenStream {
        if self.attrs.other.is_present() {
            panic!("using the other flag on an untagged variant is futile");
        }

        let ident = self.ident();

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
                let field = unnamed.iter().next().unwrap();
                quote! {
                    if let Ok(val) = #field::deserialize(reader, start, empty) {
                        return Ok(Self::#ident(val));
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    // Make sure that content is still consumed
                    if let Ok(_) = ::rustical_xml::Unit::deserialize(reader, start, empty) {
                        return Ok(Self::#ident);
                    }
                }
            }
        }
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
            .map(|variant| variant.untagged_branch());

        quote! {
            impl #impl_generics ::rustical_xml::XmlDeserialize for #name #type_generics #where_clause {
                fn deserialize<R: std::io::BufRead>(
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

        let variant_branches = self.variants.iter().map(Variant::tagged_branch);

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
}
