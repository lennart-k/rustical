use super::{
    attrs::{FieldAttrs, FieldType},
    get_generic_type,
};
use darling::FromField;
use heck::ToKebabCase;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;

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
    pub field_num: usize,
    pub attrs: FieldAttrs,
}

impl Field {
    pub fn from_syn_field(field: syn::Field, field_num: usize) -> Self {
        Self {
            attrs: FieldAttrs::from_field(&field).unwrap(),
            field,
            field_num,
        }
    }

    /// Field name in XML
    pub fn xml_name(&self) -> syn::LitByteStr {
        self.attrs.common.rename.to_owned().unwrap_or({
            let ident = self
                .field_ident()
                .as_ref()
                .expect("unnamed tag fields need a rename attribute");
            syn::LitByteStr::new(ident.to_string().to_kebab_case().as_bytes(), ident.span())
        })
    }

    /// Field identifier
    pub fn field_ident(&self) -> &Option<syn::Ident> {
        &self.field.ident
    }

    /// Builder field identifier, unnamed fields also get an identifier
    pub fn builder_field_ident(&self) -> syn::Ident {
        self.field_ident().to_owned().unwrap_or(syn::Ident::new(
            &format!("_{i}", i = self.field_num),
            self.field.span(),
        ))
    }

    pub fn target_field_index(&self) -> proc_macro2::TokenStream {
        self.field_ident()
            .as_ref()
            .map(syn::Ident::to_token_stream)
            .unwrap_or(syn::Index::from(self.field_num).to_token_stream())
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
        let builder_field_ident = self.builder_field_ident();
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

        quote! { #builder_field_ident: #builder_field_type }
    }

    /// Field initialiser in the builder struct for the deserializer
    pub fn builder_field_init(&self) -> proc_macro2::TokenStream {
        let builder_field_ident = self.builder_field_ident();
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
        quote! { #builder_field_ident: #builder_field_initialiser }
    }

    /// Map builder field to target field
    pub fn builder_field_build(&self) -> proc_macro2::TokenStream {
        // If named: use field_ident, if unnamed: use field_num
        let target_field_index = self.target_field_index();
        let builder_field_ident = self.builder_field_ident();
        let builder_value = match (
            self.attrs.flatten.is_present(),
            self.attrs.default.is_some(),
            self.is_optional(),
        ) {
            (true, _, true) => unreachable!(),
            (true, _, false) => {
                quote! { FromIterator::from_iter(builder.#builder_field_ident.into_iter()) }
            }
            (false, true, true) => unreachable!(),
            (false, true, false) => quote! { builder.#builder_field_ident },
            (false, false, true) => quote! { builder.#builder_field_ident },
            (false, false, false) => {
                let field_ident = self.field_ident().into_token_stream().to_string();
                quote! { builder.#builder_field_ident.ok_or(::rustical_xml::XmlError::MissingField(#field_ident))? }
            }
        };
        quote! { #target_field_index: #builder_value }
    }

    pub fn named_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::Tag {
            return None;
        }

        let namespace_match = if self.attrs.common.ns.is_some() {
            quote! {quick_xml::name::ResolveResult::Bound(ns)}
        } else {
            quote! {quick_xml::name::ResolveResult::Unbound}
        };

        let namespace_condition = self
            .attrs
            .common
            .ns
            .as_ref()
            .map(|ns| quote! { if ns == #ns });

        let field_name = self.xml_name();
        let builder_field_ident = self.builder_field_ident();
        let deserializer = self.deserializer_type();
        let value = quote! { <#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)? };
        let assignment = match (self.attrs.flatten.is_present(), &self.attrs.default) {
            (true, _) => {
                quote! { builder.#builder_field_ident.push(#value); }
            }
            (false, Some(_default)) => quote! { builder.#builder_field_ident = #value; },
            (false, None) => quote! { builder.#builder_field_ident = Some(#value); },
        };

        Some(quote! {
            (#namespace_match, #field_name) #namespace_condition => { #assignment; }
        })
    }

    pub fn untagged_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::Untagged {
            return None;
        }
        let builder_field_ident = self.builder_field_ident();
        let deserializer = self.deserializer_type();
        let value = quote! { <#deserializer as rustical_xml::XmlDeserialize>::deserialize(reader, &start, empty)? };

        Some(if self.attrs.flatten.is_present() {
            quote! {
                _ => { builder.#builder_field_ident.push(#value); }
            }
        } else {
            quote! {
                _ => { builder.#builder_field_ident = Some(#value); }
            }
        })
    }

    pub fn text_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::Text {
            return None;
        }
        let builder_field_ident = self.builder_field_ident();
        let value = wrap_option_if_no_default(
            quote! {
                ::rustical_xml::ValueDeserialize::deserialize(text.as_ref())?
            },
            self.attrs.default.is_some(),
        );
        Some(quote! {
            builder.#builder_field_ident = #value;
        })
    }

    pub fn attr_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::Attr {
            return None;
        }
        let builder_field_ident = self.builder_field_ident();
        let field_name = self.xml_name();

        let value = wrap_option_if_no_default(
            quote! {
            ::rustical_xml::ValueDeserialize::deserialize(attr.unescape_value()?.as_ref())?
                },
            self.attrs.default.is_some(),
        );

        Some(quote! {
            #field_name => {
                builder.#builder_field_ident = #value;
            }
        })
    }

    pub fn tagname_branch(&self) -> Option<proc_macro2::TokenStream> {
        if self.attrs.xml_ty != FieldType::TagName {
            return None;
        }
        let builder_field_ident = self.builder_field_ident();

        let value = wrap_option_if_no_default(
            quote! {
            rustical_xml::ValueDeserialize::deserialize(&String::from_utf8_lossy(name.as_ref()))?
                    },
            self.attrs.default.is_some(),
        );

        Some(quote! {
            builder.#builder_field_ident = #value;
        })
    }

    pub fn tag_writer(&self) -> Option<proc_macro2::TokenStream> {
        let target_field_index = self.target_field_index();
        let serializer = if let Some(serialize_with) = &self.attrs.serialize_with {
            quote! { #serialize_with }
        } else {
            quote! { ::rustical_xml::XmlSerialize::serialize }
        };
        let ns = match &self.attrs.common.ns {
            Some(ns) => quote! { Some(#ns) },
            None => quote! { None },
        };

        match (&self.attrs.xml_ty, self.attrs.flatten.is_present()) {
            (FieldType::Attr, _) => None,
            (FieldType::Text, true) => Some(quote! {
                for item in self.#target_field_index.iter() {
                    writer.write_event(Event::Text(BytesText::new(item)))?;
                }
            }),
            (FieldType::Text, false) => Some(quote! {
                writer.write_event(Event::Text(BytesText::new(&self.#target_field_index)))?;
            }),
            (FieldType::Tag, true) => {
                let field_name = self.xml_name();
                Some(quote! {
                    for item in self.#target_field_index.iter() {
                        #serializer(item, #ns, Some(#field_name), namespaces, writer)?;
                    }
                })
            }
            (FieldType::Tag, false) => {
                let field_name = self.xml_name();
                Some(quote! {
                    #serializer(&self.#target_field_index, #ns, Some(#field_name), namespaces, writer)?;
                })
            }
            (FieldType::Untagged, true) => Some(quote! {
                for item in self.#target_field_index.iter() {
                    #serializer(item, None, None, namespaces, writer)?;
                }
            }),
            (FieldType::Untagged, false) => Some(quote! {
                #serializer(&self.#target_field_index, None, None, namespaces, writer)?;
            }),
            // We ignore this :)
            (FieldType::TagName | FieldType::Namespace, _) => None,
        }
    }
}
