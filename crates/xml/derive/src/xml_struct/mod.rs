use crate::Field;
use crate::attrs::StructAttrs;
use core::panic;
use darling::FromDeriveInput;
use quote::quote;
use syn::{DataStruct, DeriveInput};

mod impl_se;

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

        let fields = match &data.fields {
            syn::Fields::Named(named) => named
                .named
                .iter()
                .enumerate()
                .map(|(i, field)| Field::from_syn_field(field.to_owned(), i))
                .collect(),
            syn::Fields::Unnamed(unnamed) => unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, field)| Field::from_syn_field(field.to_owned(), i))
                .collect(),
            syn::Fields::Unit => vec![],
        };

        NamedStruct {
            fields,
            attrs,
            ident: input.ident.to_owned(),
            generics: input.generics.to_owned(),
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
                    quote! { (#ns, #prefix) }
                })
                .collect()
        } else {
            vec![]
        };

        quote! {
            impl #impl_generics ::rustical_xml::XmlRootTag for #ident #type_generics #where_clause {
                fn root_tag() -> &'static str { #root }
                fn root_ns() -> Option<::quick_xml::name::Namespace<'static>> { #ns }
                fn root_ns_prefixes() -> ::std::collections::HashMap<::quick_xml::name::Namespace<'static>, &'static str> {
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
        let namespace_field_branches = self.fields.iter().filter_map(Field::namespace_branch);

        let builder_field_builds = self.fields.iter().map(Field::builder_field_build);

        let invalid_field_branch =
            invalid_field_branch(ident, self.attrs.allow_invalid.is_present());

        quote! {
            impl #impl_generics ::rustical_xml::XmlDeserialize for #ident #type_generics #where_clause {
                fn deserialize<R: ::std::io::BufRead>(
                    reader: &mut ::quick_xml::NsReader<R>,
                    start: &::quick_xml::events::BytesStart,
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
                    #(#namespace_field_branches);*

                    for attr in start.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            #(#attr_field_branches),*
                            _ => { /* ignore */ }
                        }
                    }

                    let mut string = String::new();

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
                                    let text = bytes_text.decode()?;
                                    string.push_str(&text);
                                }
                                Event::CData(cdata) => {
                                    let text = String::from_utf8(cdata.to_vec())?;
                                    string.push_str(&text);
                                }
                                Event::GeneralRef(gref) => {
                                    if let Some(char) = gref.resolve_char_ref()? {
                                        string.push(char);
                                    } else if let Some(text) =
                                        quick_xml::escape::resolve_xml_entity(&gref.xml_content()?)
                                    {
                                        string.push_str(text);
                                    } else {
                                        return Err(XmlError::UnsupportedEvent("invalid XML ref"));
                                    }
                                }
                                Event::Decl(_) => { /* <?xml ... ?> ignore this */ }
                                Event::Comment(_) => { /* ignore */ }
                                Event::DocType(_) => { /* ignore */ }
                                Event::PI(_) => { /* ignore */ }
                                Event::End(end) => {
                                    unreachable!("Unexpected closing tag for wrong element, should be handled by quick_xml");
                                }
                            }
                        }
                    }

                    let text = string;
                    #(#text_field_branches)*

                    Ok(Self {
                        #(#builder_field_builds),*
                    })
                }
            }
        }
    }
}
