use crate::de::attrs::parse_variant_attrs;
use proc_macro2::Span;
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, FieldsUnnamed, Variant};

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
        let attrs = parse_variant_attrs(&variant.attrs);

        let variant_name = attrs.rename.unwrap_or(variant.ident.to_string());
        let variant_name = syn::LitByteStr::new(variant_name.as_bytes(), Span::call_site());
        let branch = enum_variant_branch(variant);
        // dbg!(variant.fields.to_token_stream());

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
            ) -> Result<Self, rustical_xml::XmlError> {
                use quick_xml::events::Event;

                let (_ns, name) = reader.resolve_element(start.name());


                match name.as_ref() {
                    #(#variants)*
                    name => {
                        // Handle invalid variant name
                        Err(rustical_xml::XmlError::InvalidVariant(String::from_utf8_lossy(name).to_string()))
                    }
                }
            }
        }
    }
}
