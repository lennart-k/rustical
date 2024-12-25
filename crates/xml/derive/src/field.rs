use super::{
    attrs::{ContainerAttrs, FieldAttrs, FieldType},
    get_generic_type,
};
use darling::FromField;
use heck::ToKebabCase;
use quote::quote;

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
    pub container_attrs: ContainerAttrs,
}

impl Field {
    pub fn from_syn_field(field: syn::Field, container_attrs: ContainerAttrs) -> Self {
        Self {
            attrs: FieldAttrs::from_field(&field).unwrap(),
            field,
            container_attrs,
        }
    }

    /// Field name in XML
    pub fn xml_name(&self) -> syn::LitByteStr {
        self.attrs
            .common
            .rename
            .to_owned()
            .unwrap_or(syn::LitByteStr::new(
                self.field_ident().to_string().to_kebab_case().as_bytes(),
                self.field_ident().span(),
            ))
    }

    /// Whether to enforce the correct XML namespace
    pub fn ns_strict(&self) -> bool {
        self.attrs.common.ns_strict.is_present() || self.container_attrs.ns_strict.is_present()
    }

    /// Field identifier
    pub fn field_ident(&self) -> &syn::Ident {
        self.field
            .ident
            .as_ref()
            .expect("tuple structs not supported")
    }

    fn is_optional(&self) -> bool {
        if let syn::Type::Path(syn::TypePath { path, .. }) = &self.field.ty {
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
    pub fn deserializer_type(&self) -> &syn::Type {
        if self.is_optional() {
            return get_generic_type(&self.field.ty).unwrap();
        }
        if self.attrs.flatten.is_present() {
            return get_generic_type(&self.field.ty).expect("flatten attribute only implemented for explicit generics (rustical_xml will assume the first generic as the inner type)");
        }
        &self.field.ty
    }

    /// Field in the builder struct for the deserializer
    pub fn builder_field(&self) -> proc_macro2::TokenStream {
        let field_ident = self.field_ident();
        let ty = self.deserializer_type();

        let builder_field_type = match (
            self.attrs.flatten.is_present(),
            &self.attrs.default,
            self.is_optional(),
        ) {
            (_, Some(_default), true) => panic!("default value for Option<T> doesn't make sense"),
            (_, Some(_default), false) => quote! { #ty },
            (true, None, true) => panic!("cannot flatten Option<T>"),
            (true, None, false) => quote! { Vec<#ty> },
            (false, None, _) => quote! { Option<#ty> },
        };

        quote! { #field_ident: #builder_field_type }
    }

    /// Field initialiser in the builder struct for the deserializer
    pub fn builder_field_init(&self) -> proc_macro2::TokenStream {
        let field_ident = self.field_ident();
        let builder_field_initialiser = match (
            self.attrs.flatten.is_present(),
            &self.attrs.default,
            self.is_optional(),
        ) {
            (_, Some(_), true) => unreachable!(),
            (_, Some(default), false) => quote! { #default() },
            (true, None, true) => unreachable!(),
            (true, None, false) => quote! { vec![] },
            (false, None, _) => quote! { None },
        };
        quote! { #field_ident: #builder_field_initialiser }
    }

    /// Map builder field to target field
    pub fn builder_field_build(&self) -> proc_macro2::TokenStream {
        let field_ident = self.field_ident();
        let builder_value = match (
            self.attrs.flatten.is_present(),
            self.attrs.default.is_some(),
            self.is_optional(),
        ) {
            (true, _, true) => unreachable!(),
            (true, _, false) => {
                quote! { FromIterator::from_iter(builder.#field_ident.into_iter()) }
            }
            (false, true, true) => unreachable!(),
            (false, true, false) => quote! { builder.#field_ident },
            (false, false, true) => quote! { builder.#field_ident },
            (false, false, false) => {
                quote! { builder.#field_ident.expect("todo: handle missing field") }
            }
        };
        quote! { #field_ident: #builder_value }
    }

    pub fn named_branch(&self) -> Option<proc_macro2::TokenStream> {
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

        let field_name = self.xml_name();
        let field_ident = self.field_ident();
        let deserializer = self.deserializer_type();
        let value = quote! { <#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)? };
        let assignment = match (self.attrs.flatten.is_present(), &self.attrs.default) {
            (true, _) => {
                quote! { builder.#field_ident.push(#value); }
            }
            (false, Some(_default)) => quote! { builder.#field_ident = #value; },
            (false, None) => quote! { builder.#field_ident = Some(#value); },
        };

        Some(quote! {
            (#namespace_match, #field_name) => { #assignment; }
        })
    }

    pub fn untagged_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::Untagged {
            return None;
        }
        let field_ident = self.field_ident();
        let deserializer = self.deserializer_type();
        let value = quote! { <#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)? };

        Some(if self.attrs.flatten.is_present() {
            quote! {
                _ => { builder.#field_ident.push(#value); }
            }
        } else {
            quote! {
                _ => { builder.#field_ident = Some(#value); }
            }
        })
    }

    pub fn text_branch(&self) -> Option<proc_macro2::TokenStream> {
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

    pub fn attr_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::Attr {
            return None;
        }
        let field_ident = self.field_ident();
        let field_name = self.xml_name();

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

    pub fn tagname_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::TagName {
            return None;
        }
        let field_ident = self.field_ident();

        let value = wrap_option_if_no_default(
            quote! {
            rustical_xml::Value::deserialize(&String::from_utf8_lossy(name.as_ref()))?
                    },
            self.attrs.default.is_some(),
        );

        Some(quote! {
            builder.#field_ident = #value;
        })
    }

    pub fn tag_writer(&self) -> Option<proc_macro2::TokenStream> {
        let field_ident = self.field_ident();
        let field_name = self.xml_name();

        match self.attrs.xml_ty {
            FieldType::Attr => None,
            FieldType::Text => Some(quote! {
                writer.write_event(Event::Text(BytesText::new(&self.#field_ident)))?;
            }),
            FieldType::Tag => Some(quote! {
                self.#field_ident.serialize(Some(#field_name), writer)?;
            }),
            FieldType::Untagged => Some(quote! {
                // TODO: untag!
                self.#field_ident.serialize(None, writer)?;
            }),
            // TODO: Think about what to do here
            FieldType::TagName | FieldType::Namespace => None,
        }
    }
}
