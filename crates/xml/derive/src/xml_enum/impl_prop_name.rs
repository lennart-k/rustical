use super::Enum;
use quote::quote;

impl Enum {
    pub fn impl_enum_prop_name(&self) -> proc_macro2::TokenStream {
        let unit_enum_ident = self
            .attrs
            .unit_variants_ident
            .as_ref()
            .expect("unit_variants_ident no set");
        let ident = &self.ident;

        if self.attrs.untagged.is_present() {
            let variant_branches: Vec<_> = self
                .variants
                .iter()
                .map(|variant| {
                    let variant_type = variant.deserializer_type();
                    let variant_ident = &variant.variant.ident;
                    quote! {
                        #variant_ident (<#variant_type as ::rustical_xml::PropName>::Names)
                    }
                })
                .collect();

            let variant_idents: Vec<_> = self
                .variants
                .iter()
                .map(|variant| &variant.variant.ident)
                .collect();

            let unit_to_output_branches = variant_idents.iter().map(|variant_ident| {
                quote! { #unit_enum_ident::#variant_ident(val) => val.into() }
            });

            let str_to_unit_branches = self.variants.iter().map(|variant| {
                let variant_type = variant.deserializer_type();
                let variant_ident = &variant.variant.ident;
                quote! {
                    if let Ok(name) = <#variant_type as ::rustical_xml::PropName>::Names::from_str(val) {
                        return Ok(Self::#variant_ident(name))
                    }
                }
            });

            let from_enum_to_unit_branches = variant_idents.iter().map(|variant_ident| {
                quote! { #ident::#variant_ident(val) => #unit_enum_ident::#variant_ident(val.into()) }
            });

            quote! {
                #[derive(Clone, Debug, PartialEq, Hash, Eq, ::rustical_xml::XmlDeserialize)]
                #[xml(untagged)]
                pub enum #unit_enum_ident {
                    #(#variant_branches),*
                }

                impl ::rustical_xml::PropName for #ident {
                    type Names = #unit_enum_ident;
                }

                impl From<#unit_enum_ident> for (Option<::quick_xml::name::Namespace<'static>>, &'static str) {
                    fn from(val: #unit_enum_ident) -> Self {
                        match val {
                            #(#unit_to_output_branches),*
                        }
                    }
                }

                 impl From<#ident> for #unit_enum_ident {
                     fn from(val: #ident) -> Self {
                         match val {
                             #(#from_enum_to_unit_branches),*
                         }
                     }
                 }

                 impl ::std::str::FromStr for #unit_enum_ident {
                     type Err = ::rustical_xml::FromStrError;

                     fn from_str(val: &str) -> Result<Self, Self::Err> {
                        #(#str_to_unit_branches);*
                        Err(::rustical_xml::FromStrError)
                     }
                 }
            }
        } else {
            let tagged_variants: Vec<_> = self
                .variants
                .iter()
                .filter(|variant| !variant.attrs.other.is_present())
                .collect();

            let prop_name_variants = tagged_variants.iter().map(|variant| {
                let ident = &variant.variant.ident;
                if let Some(proptype) = &variant.attrs.prop {
                    quote! {#ident(#proptype)}
                } else {
                    quote! {#ident}
                }
            });

            let unit_to_output_branches = tagged_variants.iter().map(|variant| {
                let ns = match &variant.attrs.common.ns {
                    Some(ns) => quote! { Some(#ns) },
                    None => quote! { None },
                };
                let b_xml_name = variant.xml_name().value();
                let xml_name = String::from_utf8_lossy(&b_xml_name);
                let out = quote! {(#ns, #xml_name)};

                let ident = &variant.variant.ident;
                if variant.attrs.prop.is_some() {
                    quote! { #unit_enum_ident::#ident(..) => #out }
                } else {
                    quote! { #unit_enum_ident::#ident => #out }
                }
            });

            let from_enum_to_unit_branches = tagged_variants.iter().map(|variant| {
                let variant_ident = &variant.variant.ident;
                if variant.attrs.prop.is_some() {
                    quote! { #ident::#variant_ident { .. } => Self::#variant_ident (Default::default()) }
                } else {
                    quote! { #ident::#variant_ident { .. } => Self::#variant_ident }
                }
            });

            let str_to_unit_branches = tagged_variants.iter().map(|variant| {
                let ident = &variant.variant.ident;
                let b_xml_name = variant.xml_name().value();
                let xml_name = String::from_utf8_lossy(&b_xml_name);
                if variant.attrs.prop.is_some() {
                    quote! { #xml_name => Ok(Self::#ident (Default::default())) }
                } else {
                    quote! { #xml_name => Ok(Self::#ident) }
                }
            });

            quote! {
                #[derive(Clone, Debug, PartialEq, Eq, Hash, ::rustical_xml::XmlDeserialize)]
                pub enum #unit_enum_ident {
                    #(#prop_name_variants),*
                }


                impl ::rustical_xml::PropName for #ident {
                    type Names = #unit_enum_ident;
                }

                impl From<#unit_enum_ident> for (Option<::quick_xml::name::Namespace<'static>>, &'static str) {
                    fn from(val: #unit_enum_ident) -> Self {
                        match val {
                            #(#unit_to_output_branches),*
                        }
                    }
                }

                impl From<#ident> for #unit_enum_ident {
                    fn from(val: #ident) -> Self {
                        match val {
                            #(#from_enum_to_unit_branches),*
                        }
                    }
                }

                impl ::std::str::FromStr for #unit_enum_ident {
                    type Err = ::rustical_xml::FromStrError;

                    fn from_str(val: &str) -> Result<Self, Self::Err> {
                        match val {
                            #(#str_to_unit_branches),*,
                            _ => Err(::rustical_xml::FromStrError)
                        }
                    }
                }
            }
        }
    }
}
