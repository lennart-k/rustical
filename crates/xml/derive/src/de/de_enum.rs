use darling::FromVariant;
use heck::ToKebabCase;
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, FieldsUnnamed, Variant};

use crate::de::attrs::VariantAttrs;

pub fn enum_variant_branch(variant: &Variant) -> proc_macro2::TokenStream {
    let ident = &variant.ident;

    match &variant.fields {
        Fields::Named(named) => {
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

pub fn impl_de_enum(input: &DeriveInput, data: &DataEnum) -> proc_macro2::TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;

    let variants = data.variants.iter().map(|variant| {
        let attrs = VariantAttrs::from_variant(variant).unwrap();
        let variant_name = attrs.common.rename.unwrap_or(syn::LitByteStr::new(
            variant.ident.to_string().to_kebab_case().as_bytes(),
            variant.ident.span(),
        ));
        let branch = enum_variant_branch(variant);
        dbg!(&variant_name);

        quote! {
            #variant_name => {
                #branch
            }
        }
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
                    #(#variants)*
                    name => {
                        // Handle invalid variant name
                        Err(rustical_xml::XmlDeError::InvalidVariant(String::from_utf8_lossy(name).to_string()))
                    }
                }
            }
        }
    }
}
