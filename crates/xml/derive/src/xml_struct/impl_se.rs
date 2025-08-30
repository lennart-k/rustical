use quote::quote;

use crate::{Field, attrs::FieldType};

use super::NamedStruct;

impl NamedStruct {
    pub fn impl_se(&self) -> proc_macro2::TokenStream {
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;
        let tag_writers: Vec<_> = self.fields.iter().filter_map(Field::tag_writer).collect();

        let untagged_attributes = self
            .fields
            .iter()
            .filter(|field| field.attrs.xml_ty == FieldType::Untagged)
            .filter(|field| !field.attrs.flatten.is_present())
            .map(|field| {
                let field_ident = field.target_field_index();
                quote! {
                    if let Some(attrs) = self.#field_ident.attributes() {
                        bytes_start.extend_attributes(attrs);
                    }
                }
            });

        let attributes = self
            .fields
            .iter()
            .filter(|field| field.attrs.xml_ty == FieldType::Attr)
            .map(|field| {
                let field_name = field.xml_name();
                let field_index = field.target_field_index();
                quote! {
                    ::quick_xml::events::attributes::Attribute {
                        key: ::quick_xml::name::QName(#field_name.as_bytes()),
                        value: ::std::borrow::Cow::from(::rustical_xml::ValueSerialize::serialize(&self.#field_index).into_bytes())
                    }
                }
            });

        let tag_name_field = self
            .fields
            .iter()
            .find(|field| field.attrs.xml_ty == FieldType::TagName)
            .map(|field| {
                let field_index = field.target_field_index();
                quote! {
                    let tag_str = self.#field_index.to_string();
                    let tag = Some(tag.unwrap_or(tag_str.as_str()));
                }
            });

        let namespace_field = self
            .fields
            .iter()
            .find(|field| field.attrs.xml_ty == FieldType::Namespace)
            .map(|field| {
                let field_index = field.target_field_index();
                quote! {
                    let ns = self.#field_index;
                }
            });

        let is_empty = tag_writers.is_empty();

        // If we are the root element write the xmlns attributes
        let prefix_attributes = if self.attrs.root.is_some() {
            self.attrs
                .ns_prefix
                .iter()
                .map(|(ns, prefix)| {
                    let attr_name = if prefix.value().is_empty() {
                        "xmlns".to_owned()
                    } else {
                        format!("xmlns:{}", prefix.value())
                    };
                    let a = syn::LitByteStr::new(attr_name.as_bytes(), prefix.span());
                    quote! {
                         bytes_start.push_attribute((#a.as_ref(), #ns.as_ref()));
                    }
                })
                .collect()
        } else {
            vec![]
        };

        quote! {
            impl #impl_generics ::rustical_xml::XmlSerialize for #ident #type_generics #where_clause {
                fn serialize(
                    &self,
                    ns: Option<::quick_xml::name::Namespace>,
                    tag: Option<&str>,
                    namespaces: &::std::collections::HashMap<::quick_xml::name::Namespace, &str>,
                    writer: &mut ::quick_xml::Writer<&mut Vec<u8>>
                ) -> ::std::io::Result<()> {
                    use ::quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

                    #tag_name_field;
                    #namespace_field;

                     let prefix = ns
                         .map(|ns| namespaces.get(&ns))
                         .unwrap_or(None)
                         .map(|prefix| {
                             if !prefix.is_empty() {
                                format!("{prefix}:")
                             } else {
                                String::new()
                             }
                         });
                     let has_prefix = prefix.is_some();
                     let tagname = tag.map(|tag| [&prefix.unwrap_or_default(), tag].concat());
                     let qname = tagname.as_ref().map(|tagname| ::quick_xml::name::QName(tagname.as_bytes()));

                    if let Some(qname) = &qname {
                        let mut bytes_start = BytesStart::from(qname.to_owned());
                        if !has_prefix {
                            if let Some(ns) = &ns {
                                bytes_start.push_attribute((b"xmlns".as_ref(), ns.as_ref()));
                            }
                        }
                        #(#prefix_attributes);*
                        if let Some(attrs) = self.attributes() {
                            bytes_start.extend_attributes(attrs);
                        }
                        #(#untagged_attributes);*
                        if #is_empty {
                            writer.write_event(Event::Empty(bytes_start))?;
                        } else {
                            writer.write_event(Event::Start(bytes_start))?;
                        }
                    }
                    if !#is_empty {
                        #(#tag_writers);*
                        if let Some(qname) = &qname {
                            writer.write_event(Event::End(BytesEnd::from(qname.to_owned())))?;
                        }
                    }
                    Ok(())
                }

                fn attributes<'a>(&self) -> Option<Vec<::quick_xml::events::attributes::Attribute<'a>>> {
                    Some(vec![ #(#attributes),* ])
                }
            }
        }
    }
}
