use crate::attrs::{FieldType, StructAttrs};
use crate::Field;
use core::panic;
use darling::FromDeriveInput;
use quote::quote;
use syn::{DataStruct, DeriveInput};

fn invalid_field_branch(ident: &syn::Ident, allow: bool) -> proc_macro2::TokenStream {
    let ident = ident.to_string();
    if allow {
        quote! {}
    } else {
        quote! {
        return Err(XmlError::InvalidFieldName(#ident, format!("[{ns:?}]{tag}", tag = String::from_utf8_lossy(tag)))) }
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
                    .enumerate()
                    .map(|(i, field)| Field::from_syn_field(field.to_owned(), i))
                    .collect(),
                attrs,
                ident: input.ident.to_owned(),
                generics: input.generics.to_owned(),
            },
            syn::Fields::Unnamed(unnamed) => NamedStruct {
                fields: unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, field)| Field::from_syn_field(field.to_owned(), i))
                    .collect(),
                attrs,
                ident: input.ident.to_owned(),
                generics: input.generics.to_owned(),
            },

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
    pub fn impl_xml_root_tag(&self) -> proc_macro2::TokenStream {
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;
        let root = self.attrs.root.as_ref().expect("No root attribute found");
        let ns = match &self.attrs.ns {
            Some(ns) => quote! { Some(#ns) },
            None => quote! { None },
        };

        let prefixes = if self.attrs.root.is_some() {
            self.attrs
                .ns_prefix
                .iter()
                .map(|(ns, prefix)| {
                    quote! { (#ns, #prefix.as_ref()) }
                })
                .collect()
        } else {
            vec![]
        };

        quote! {
            impl #impl_generics ::rustical_xml::XmlRootTag for #ident #type_generics #where_clause {
                fn root_tag() -> &'static [u8] { #root }
                fn root_ns() -> Option<::quick_xml::name::Namespace<'static>> { #ns }
                fn root_ns_prefixes() -> ::std::collections::HashMap<::quick_xml::name::Namespace<'static>, &'static [u8]> {
                    ::std::collections::HashMap::from_iter(vec![
                        #(#prefixes),*
                    ])
                }
            }
        }
    }

    pub fn impl_de(&self) -> proc_macro2::TokenStream {
        let builder_generics = &self.generics;
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
        let text_field_branches: Vec<_> =
            self.fields.iter().filter_map(Field::text_branch).collect();
        let attr_field_branches = self.fields.iter().filter_map(Field::attr_branch);
        let tagname_field_branches = self.fields.iter().filter_map(Field::tagname_branch);

        let builder_field_builds = self.fields.iter().map(Field::builder_field_build);

        let invalid_field_branch =
            invalid_field_branch(ident, self.attrs.allow_invalid.is_present());

        quote! {
            impl #impl_generics ::rustical_xml::XmlDeserialize for #ident #type_generics #where_clause {
                fn deserialize<R: ::std::io::BufRead>(
                    reader: &mut quick_xml::NsReader<R>,
                    start: &quick_xml::events::BytesStart,
                    empty: bool
                ) -> Result<Self, rustical_xml::XmlError> {
                    use quick_xml::events::Event;
                    use rustical_xml::XmlError;

                    let mut buf = Vec::new();

                    // initialise fields
                    struct StructBuilder #builder_generics {
                        #(#builder_fields),*
                    }

                    let mut builder = StructBuilder {
                        #(#builder_field_inits),*
                    };

                    let (ns, name) = reader.resolve_element(start.name());
                    #(#tagname_field_branches);*

                    for attr in start.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            #(#attr_field_branches),*
                            _ => { /* ignore */ }
                        }
                    }

                    if !empty {
                        loop {
                            let event = reader.read_event_into(&mut buf)?;
                            match &event {
                                Event::End(e) if e.name() == start.name() => {
                                    break;
                                }
                                Event::Eof => return Err(XmlError::Eof),
                                // start of a child element
                                Event::Start(start) | Event::Empty(start) => {
                                    let empty = matches!(event, Event::Empty(_));
                                    let (ns, name) = reader.resolve_element(start.name());
                                    match (ns, name.as_ref()) {
                                        #(#named_field_branches),*
                                        #(#untagged_field_branches),*
                                        (ns, tag) => { #invalid_field_branch }
                                    }
                                }
                                Event::Text(bytes_text) => {
                                    let text = bytes_text.unescape()?;
                                    #(#text_field_branches)*
                                }
                                Event::CData(cdata) => {
                                    let text = String::from_utf8(cdata.to_vec())?;
                                    #(#text_field_branches)*
                                }
                                Event::Decl(_) => { /* <?xml ... ?> ignore this */ }
                                Event::Comment(_) => { /* ignore */ }
                                Event::DocType(_) => { /* ignore */ }
                                Event::PI(_) => {
                                    // Error: not supported
                                    return Err(XmlError::UnsupportedEvent("Processing instruction"));
                                }
                                Event::End(end) => {
                                    unreachable!("Unexpected closing tag for wrong element, should be handled by quick_xml");
                                }
                            }
                        }
                    }

                    Ok(Self {
                        #(#builder_field_builds),*
                    })
                }
            }
        }
    }

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
                let field_ident = field.field_ident();
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
                        key: ::quick_xml::name::QName(#field_name),
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
                    let tag = Some(tag.unwrap_or(tag_str.as_bytes()));
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
                    let sep = if !prefix.value().is_empty() {
                        b":".to_vec()
                    } else {
                        b"".to_vec()
                    };
                    let attr_name = [b"xmlns".as_ref(), &sep, &prefix.value()].concat();
                    let a = syn::LitByteStr::new(&attr_name, prefix.span());
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
                fn serialize<W: ::std::io::Write>(
                    &self,
                    ns: Option<::quick_xml::name::Namespace>,
                    tag: Option<&[u8]>,
                    namespaces: &::std::collections::HashMap<::quick_xml::name::Namespace, &[u8]>,
                    writer: &mut ::quick_xml::Writer<W>
                ) -> ::std::io::Result<()> {
                    use ::quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

                    #tag_name_field;
                    #namespace_field;

                    let prefix = ns
                        .map(|ns| namespaces.get(&ns))
                        .unwrap_or(None)
                        .map(|prefix| {
                            if !prefix.is_empty() {
                                [*prefix, b":"].concat()
                            } else {
                                Vec::new()
                            }
                        });
                    let has_prefix = prefix.is_some();
                    let tagname = tag.map(|tag| [&prefix.unwrap_or_default(), tag].concat());
                    let qname = tagname.as_ref().map(|tagname| ::quick_xml::name::QName(tagname));

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
