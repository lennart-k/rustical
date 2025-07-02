use quote::quote;

use crate::Variant;

use super::Enum;

impl Enum {
    pub fn impl_se(&self) -> proc_macro2::TokenStream {
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;
        let enum_untagged = self.attrs.untagged.is_present();
        let variant_serializers = self.variants.iter().map(Variant::se_branch);

        quote! {
            impl #impl_generics ::rustical_xml::XmlSerialize for #ident #type_generics #where_clause {
                fn serialize(
                    &self,
                    ns: Option<::quick_xml::name::Namespace>,
                    tag: Option<&[u8]>,
                    namespaces: &::std::collections::HashMap<::quick_xml::name::Namespace, &[u8]>,
                    writer: &mut ::quick_xml::Writer<&mut [u8]>
                ) -> ::std::io::Result<()> {
                    use ::quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

                    let prefix = ns
                        .map(|ns| namespaces.get(&ns))
                        .unwrap_or(None)
                        .map(|prefix| if !prefix.is_empty() {
                            [*prefix, b":"].concat()
                        } else {
                            vec![]
                        });
                    let has_prefix = prefix.is_some();
                    let tagname = tag.map(|tag| [&prefix.unwrap_or_default(), tag].concat());
                    let qname = tagname.as_ref().map(|tagname| ::quick_xml::name::QName(tagname));

                    const enum_untagged: bool = #enum_untagged;

                    if let Some(qname) = &qname {
                        let mut bytes_start = BytesStart::from(qname.to_owned());
                        if !has_prefix {
                            if let Some(ns) = &ns {
                                bytes_start.push_attribute((b"xmlns".as_ref(), ns.as_ref()));
                            }
                        }
                        writer.write_event(Event::Start(bytes_start))?;
                    }

                    #(#variant_serializers);*

                    if let Some(qname) = &qname {
                        writer.write_event(Event::End(BytesEnd::from(qname.to_owned())))?;
                    }
                    Ok(())
                }

                fn attributes<'a>(&self) -> Option<Vec<::quick_xml::events::attributes::Attribute<'a>>> {
                    None
                }
            }
        }
    }
}
