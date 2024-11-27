use crate::de::attrs::StructAttrs;

use super::attrs::FieldAttrs;
use core::panic;
use darling::{FromDeriveInput, FromField};
use heck::ToKebabCase;
use quote::quote;
use syn::{AngleBracketedGenericArguments, DataStruct, DeriveInput, LitByteStr, TypePath};

fn get_generic_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            if let syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                args, ..
            }) = &seg.arguments
            {
                if let Some(syn::GenericArgument::Type(t)) = &args.first() {
                    return Some(t);
                }
            }
        }
    }
    None
}

pub struct Field {
    pub field: syn::Field,
    pub attrs: FieldAttrs,
}

impl Field {
    fn from_syn_field(field: syn::Field) -> Self {
        Self {
            attrs: FieldAttrs::from_field(&field).unwrap(),
            field,
        }
    }
    fn de_name(&self) -> LitByteStr {
        self.attrs
            .common
            .rename
            .to_owned()
            .unwrap_or(LitByteStr::new(
                self.field_ident().to_string().to_kebab_case().as_bytes(),
                self.field_ident().span(),
            ))
    }

    fn field_ident(&self) -> &syn::Ident {
        self.field
            .ident
            .as_ref()
            .expect("tuple structs not supported")
    }

    fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    fn builder_field(&self) -> proc_macro2::TokenStream {
        let field_ident = self.field_ident();
        let ty = self.ty();
        if self.attrs.default.is_some() {
            quote! {
                #field_ident: #ty,
            }
        } else if self.attrs.flatten.is_present() {
            let generic_type = get_generic_type(ty).expect("flatten attribute only implemented for explicit generics (rustical_xml will assume the first generic as the inner type)");
            quote! {
                #field_ident: Vec<#generic_type>,
            }
        } else {
            quote! {
                #field_ident: Option<#ty>,
            }
        }
    }

    fn builder_field_init(&self) -> proc_macro2::TokenStream {
        let field_ident = self.field_ident();
        if let Some(default) = &self.attrs.default {
            quote! {
                #field_ident: #default(),
            }
        } else if self.attrs.flatten.is_present() {
            quote! {
                #field_ident: vec![],
            }
        } else {
            quote! {
                #field_ident: None,
            }
        }
    }

    fn builder_field_build(&self) -> proc_macro2::TokenStream {
        let field_ident = self.field_ident();
        if self.attrs.flatten.is_present() {
            quote! {
                #field_ident: FromIterator::from_iter(builder.#field_ident.into_iter())
            }
        } else if self.attrs.default.is_some() {
            quote! {
                #field_ident: builder.#field_ident,
            }
        } else {
            quote! {
                #field_ident: builder.#field_ident.expect("todo: handle missing field"),
            }
        }
    }

    fn named_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.text.is_present() {
            return None;
        }
        if self.attrs.untagged.is_present() {
            return None;
        }
        let field_name = self.de_name();
        let field_ident = self.field_ident();
        let deserializer = self.ty();
        Some(if self.attrs.default.is_some() {
            quote! {
                #field_name => {
                     builder.#field_ident = <#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)?;
                },
            }
        } else if self.attrs.flatten.is_present() {
            let deserializer = get_generic_type(self.ty()).expect("flatten attribute only implemented for explicit generics (rustical_xml will assume the first generic as the inner type)");
            quote! {
                #field_name => {
                     builder.#field_ident.push(<#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)?);
                },
            }
        } else {
            quote! {
                #field_name => {
                     builder.#field_ident = Some(<#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)?);
                },
            }
        })
    }

    fn untagged_branch(&self) -> Option<proc_macro2::TokenStream> {
        if !self.attrs.untagged.is_present() {
            return None;
        }
        let field_ident = self.field_ident();
        let deserializer = self.ty();
        Some(if self.attrs.flatten.is_present() {
            let deserializer = get_generic_type(self.ty()).expect("flatten attribute only implemented for explicit generics (rustical_xml will assume the first generic as the inner type)");
            quote! {
                _ => {
                     builder.#field_ident.push(<#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)?);
                },
            }
        } else {
            quote! {
                _ => {
                     builder.#field_ident = Some(<#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)?);
                },
            }
        })
    }

    fn text_branch(&self) -> Option<proc_macro2::TokenStream> {
        if !self.attrs.text.is_present() {
            return None;
        }
        let field_ident = self.field_ident();
        Some(quote! {
            builder.#field_ident = Some(text.to_owned().into());
        })
    }
}

pub fn impl_de_struct(input: &DeriveInput, data: &DataStruct) -> proc_macro2::TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;

    let struct_attrs = StructAttrs::from_derive_input(input).unwrap();

    let fields: Vec<_> = data
        .fields
        .iter()
        .map(|field| Field::from_syn_field(field.to_owned()))
        .collect();

    let builder_fields = fields.iter().map(Field::builder_field);
    let builder_field_inits = fields.iter().map(Field::builder_field_init);
    let named_field_branches = fields.iter().filter_map(Field::named_branch);
    let untagged_field_branches: Vec<_> =
        fields.iter().filter_map(Field::untagged_branch).collect();
    if untagged_field_branches.len() > 1 {
        panic!("Currently only one untagged field supported!");
    }
    let text_field_branches = fields.iter().filter_map(Field::text_branch);
    let builder_field_builds = fields.iter().map(Field::builder_field_build);

    let xml_root_impl = if let Some(root) = struct_attrs.root {
        quote! {
            impl #impl_generics ::rustical_xml::XmlRoot for #name #type_generics #where_clause {
                fn root_tag() -> &'static [u8] {
                    #root
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #xml_root_impl

        impl #impl_generics ::rustical_xml::XmlDeserialize for #name #type_generics #where_clause {
            fn deserialize<R: std::io::BufRead>(
                reader: &mut quick_xml::NsReader<R>,
                start: &quick_xml::events::BytesStart,
                empty: bool
            ) -> Result<Self, rustical_xml::XmlDeError> {
                use quick_xml::events::Event;
                use rustical_xml::XmlDeError;

                 let mut buf = Vec::new();

                // initialise fields
                struct StructBuilder {
                    #(#builder_fields)*
                }

                let mut builder = StructBuilder {
                    #(#builder_field_inits)*
                };

                if !empty {
                    loop {
                        let event = reader.read_event_into(&mut buf)?;
                        match &event {
                            Event::End(e) if e.name() == start.name() => {
                                break;
                            }
                            Event::Eof => return Err(XmlDeError::Eof),
                            // start of a child element
                            Event::Start(start) | Event::Empty(start) => {
                                let empty = matches!(event, Event::Empty(_));
                                let (_ns, name) = reader.resolve_element(start.name());
                                match name.as_ref() {
                                    #(#named_field_branches)*
                                    #(#untagged_field_branches)*
                                    _ => {
                                        // invalid field name
                                        return Err(XmlDeError::InvalidFieldName)
                                    }
                                }
                            }
                            Event::Text(bytes_text) => {
                                let text = bytes_text.unescape()?;
                                #(#text_field_branches)*
                            }
                            Event::CData(cdata) => {
                                return Err(XmlDeError::UnsupportedEvent("CDATA"));
                            }
                            Event::Comment(_) => {
                                // ignore
                            }
                            Event::Decl(_) => {
                                // Error: not supported
                                return Err(XmlDeError::UnsupportedEvent("Declaration"));
                            }
                            Event::PI(_) => {
                                // Error: not supported
                                return Err(XmlDeError::UnsupportedEvent("Processing instruction"));
                            }
                            Event::DocType(doctype) => {
                                // Error: start of new document
                                return Err(XmlDeError::UnsupportedEvent("Doctype in the middle of the document"));
                            }
                            Event::End(end) => {
                                // Error: premature end
                                return Err(XmlDeError::Other("Unexpected closing tag for wrong element".to_owned()));
                            }
                        }
                    }
                }

                Ok(Self {
                    #(#builder_field_builds)*
                })
            }
        }
    }
}
