use super::attrs::{FieldAttrs, FieldType};
use crate::de::attrs::StructAttrs;
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

fn invalid_field_branch(allow: bool) -> proc_macro2::TokenStream {
    if allow {
        quote! {
            _ => {
                // ignore because of allow_invalid flag
            }
        }
    } else {
        quote! {
            _ => {
                // invalid field name
                return Err(XmlDeError::InvalidFieldName)
            }
        }
    }
}

fn wrap_option_if_no_default(
    value: proc_macro2::TokenStream,
    has_default: bool,
) -> proc_macro2::TokenStream {
    if has_default {
        value
    } else {
        quote! {Some(#value)}
    }
}

pub struct Field {
    pub field: syn::Field,
    pub attrs: FieldAttrs,
    pub struct_attrs: StructAttrs,
}

impl Field {
    fn from_syn_field(field: syn::Field, struct_attrs: StructAttrs) -> Self {
        Self {
            attrs: FieldAttrs::from_field(&field).unwrap(),
            field,
            struct_attrs,
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

    fn ns_strict(&self) -> bool {
        self.attrs.common.ns_strict.is_present()
            || self.struct_attrs.container.ns_strict.is_present()
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
        match (self.attrs.flatten.is_present(), &self.attrs.default) {
            (_, Some(_default)) => quote! { #field_ident: #ty, },
            (true, None) => {
                let generic_type = get_generic_type(ty).expect("flatten attribute only implemented for explicit generics (rustical_xml will assume the first generic as the inner type)");
                quote! { #field_ident: Vec<#generic_type>, }
            }
            (false, None) => quote! { #field_ident: Option<#ty>, },
        }
    }

    fn builder_field_init(&self) -> proc_macro2::TokenStream {
        let field_ident = self.field_ident();
        match (self.attrs.flatten.is_present(), &self.attrs.default) {
            (_, Some(default)) => quote! { #field_ident: #default(), },
            (true, None) => quote! { #field_ident: vec![], },
            (false, None) => quote! { #field_ident: None, },
        }
    }

    fn builder_field_build(&self) -> proc_macro2::TokenStream {
        let field_ident = self.field_ident();
        match (
            self.attrs.flatten.is_present(),
            self.attrs.default.is_some(),
        ) {
            (true, _) => quote! {
                #field_ident: FromIterator::from_iter(builder.#field_ident.into_iter())
            },
            (false, true) => quote! {
                #field_ident: builder.#field_ident,
            },
            (false, false) => quote! {
                #field_ident: builder.#field_ident.expect("todo: handle missing field"),
            },
        }
    }

    fn named_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::Tag {
            return None;
        }

        let namespace_match = if self.ns_strict() {
            if let Some(ns) = &self.attrs.common.ns {
                quote! {quick_xml::name::ResolveResult::Bound(quick_xml::name::Namespace(#ns))}
            } else {
                quote! {quick_xml::name::ResolveResult::Unbound}
            }
        } else {
            quote! {_}
        };

        let field_name = self.de_name();
        let field_ident = self.field_ident();
        let deserializer = self.ty();
        let value = quote! { <#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)? };
        let assignment = match (self.attrs.flatten.is_present(), &self.attrs.default) {
            (true, _) => {
                // TODO: Make nicer, watch out with deserializer typing
                let deserializer = get_generic_type(self.ty()).expect("flatten attribute only implemented for explicit generics (rustical_xml will assume the first generic as the inner type)");
                quote! {
                    builder.#field_ident.push(<#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)?);
                }
            }
            (false, Some(_default)) => quote! {
                builder.#field_ident = #value;
            },
            (false, None) => quote! {
                builder.#field_ident = Some(#value);
            },
        };

        Some(quote! {
            (#namespace_match, #field_name) => { #assignment; },
        })
    }

    fn untagged_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::Untagged {
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
        if self.attrs.xml_ty != FieldType::Text {
            return None;
        }
        let field_ident = self.field_ident();
        let value = wrap_option_if_no_default(
            quote! {
            rustical_xml::Value::deserialize(text.as_ref())?
                    },
            self.attrs.default.is_some(),
        );
        Some(quote! {
            builder.#field_ident = #value;
        })
    }

    fn attr_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::Attr {
            return None;
        }
        let field_ident = self.field_ident();
        let field_name = self.de_name();

        let value = wrap_option_if_no_default(
            quote! {
            rustical_xml::Value::deserialize(attr.unescape_value()?.as_ref())?
                    },
            self.attrs.default.is_some(),
        );

        Some(quote! {
            #field_name => {
                builder.#field_ident = #value;
            }
        })
    }
}

impl NamedStruct {
    pub fn parse(input: &DeriveInput, data: &DataStruct) -> Self {
        let attrs = StructAttrs::from_derive_input(input).unwrap();

        match &data.fields {
            syn::Fields::Named(named) => NamedStruct {
                fields: named
                    .named
                    .iter()
                    .map(|field| Field::from_syn_field(field.to_owned(), attrs.clone()))
                    .collect(),
                attrs,
                ident: input.ident.to_owned(),
                generics: input.generics.to_owned(),
            },
            syn::Fields::Unnamed(_) => panic!("not implemented for tuple struct"),
            syn::Fields::Unit => NamedStruct {
                fields: vec![],
                attrs,
                ident: input.ident.to_owned(),
                generics: input.generics.to_owned(),
            },
        }
    }
}

pub struct NamedStruct {
    attrs: StructAttrs,
    fields: Vec<Field>,
    ident: syn::Ident,
    generics: syn::Generics,
}

impl NamedStruct {
    pub fn impl_de(&self) -> proc_macro2::TokenStream {
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;

        let builder_fields = self.fields.iter().map(Field::builder_field);
        let builder_field_inits = self.fields.iter().map(Field::builder_field_init);
        let named_field_branches = self.fields.iter().filter_map(Field::named_branch);
        let untagged_field_branches: Vec<_> = self
            .fields
            .iter()
            .filter_map(Field::untagged_branch)
            .collect();
        if untagged_field_branches.len() > 1 {
            panic!("Currently only one untagged field supported!");
        }
        let text_field_branches = self.fields.iter().filter_map(Field::text_branch);
        let attr_field_branches = self.fields.iter().filter_map(Field::attr_branch);

        let builder_field_builds = self.fields.iter().map(Field::builder_field_build);

        let xml_root_impl = if let Some(root) = &self.attrs.root {
            quote! {
                impl #impl_generics ::rustical_xml::XmlRoot for #ident #type_generics #where_clause {
                    fn root_tag() -> &'static [u8] { #root }
                }
            }
        } else {
            quote! {}
        };

        let invalid_field_branch = invalid_field_branch(self.attrs.allow_invalid.is_present());

        quote! {
            #xml_root_impl

            impl #impl_generics ::rustical_xml::XmlDeserialize for #ident #type_generics #where_clause {
                fn deserialize<R: BufRead>(
                    reader: &mut quick_xml::NsReader<R>,
                    start: &quick_xml::events::BytesStart,
                    empty: bool
                ) -> Result<Self, rustical_xml::XmlDeError> {
                    use quick_xml::events::Event;
                    use rustical_xml::XmlDeError;

                    let mut buf = Vec::new();

                    // initialise fields
                    struct StructBuilder #type_generics #where_clause {
                        #(#builder_fields)*
                    }

                    let mut builder = StructBuilder {
                        #(#builder_field_inits)*
                    };

                    for attr in start.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            #(#attr_field_branches)*
                            #invalid_field_branch
                        }
                    }

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
                                    let (ns, name) = reader.resolve_element(start.name());
                                    match (ns, name.as_ref()) {
                                        #(#named_field_branches)*
                                        #(#untagged_field_branches)*
                                        #invalid_field_branch
                                    }
                                }
                                Event::Text(bytes_text) => {
                                    let text = bytes_text.unescape()?;
                                    #(#text_field_branches)*
                                }
                                Event::CData(cdata) => {
                                    return Err(XmlDeError::UnsupportedEvent("CDATA"));
                                }
                                Event::Comment(_) => { /* ignore */ }
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
}
