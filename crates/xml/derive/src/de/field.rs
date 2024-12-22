use super::attrs::{ContainerAttrs, FieldAttrs, FieldType};
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

fn get_generic_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                args,
                ..
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
    pub fn de_name(&self) -> syn::LitByteStr {
        self.attrs
            .common
            .rename
            .to_owned()
            .unwrap_or(syn::LitByteStr::new(
                self.field_ident().to_string().to_kebab_case().as_bytes(),
                self.field_ident().span(),
            ))
    }

    pub fn ns_strict(&self) -> bool {
        self.attrs.common.ns_strict.is_present() || self.container_attrs.ns_strict.is_present()
    }

    pub fn field_ident(&self) -> &syn::Ident {
        self.field
            .ident
            .as_ref()
            .expect("tuple structs not supported")
    }

    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    pub fn builder_field(&self) -> proc_macro2::TokenStream {
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

    pub fn builder_field_init(&self) -> proc_macro2::TokenStream {
        let field_ident = self.field_ident();
        match (self.attrs.flatten.is_present(), &self.attrs.default) {
            (_, Some(default)) => quote! { #field_ident: #default(), },
            (true, None) => quote! { #field_ident: vec![], },
            (false, None) => quote! { #field_ident: None, },
        }
    }

    pub fn builder_field_build(&self) -> proc_macro2::TokenStream {
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

    pub fn untagged_branch(&self) -> Option<proc_macro2::TokenStream> {
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
