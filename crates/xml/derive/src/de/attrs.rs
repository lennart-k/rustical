use core::panic;

use heck::{ToKebabCase, ToPascalCase};
use quote::ToTokens;
use strum::EnumString;
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Expr, ExprLit, Lit, LitByteStr, LitStr, Meta,
};

const ATTR_SCOPE: &str = "xml";

#[derive(Default)]
pub struct VariantAttrs {
    pub rename: Option<String>,
    pub ns: Option<String>,
}

pub fn get_scoped_attrs(attrs: &[Attribute]) -> Option<Punctuated<Meta, Comma>> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident(ATTR_SCOPE))
        .map(|attr| {
            attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                .unwrap()
        })
}

pub fn parse_variant_attrs(attrs: &[Attribute]) -> VariantAttrs {
    let mut variant_attrs = VariantAttrs::default();

    let attrs = get_scoped_attrs(attrs);

    let attrs = if let Some(attrs) = attrs {
        attrs
    } else {
        return variant_attrs;
    };

    for meta in attrs {
        match meta {
            // single flag
            Meta::Path(path) => {
                panic!("unrecognized variant flag: {}", path.to_token_stream());
            }
            Meta::List(list) => {
                panic!("list-type attrs not supported: {}", list.to_token_stream());
            }
            Meta::NameValue(name_value) => {
                if name_value.path.is_ident("ns") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = name_value.value
                    {
                        variant_attrs.ns = Some(lit_str.value());
                    }
                } else if name_value.path.is_ident("rename") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = name_value.value
                    {
                        variant_attrs.rename = Some(lit_str.value());
                    }
                } else {
                    panic!(
                        "unrecognized variant attribute: {}",
                        name_value.to_token_stream()
                    );
                }
            }
        }
    }
    variant_attrs
}

#[derive(EnumString)]
pub enum CaseStyle {
    #[strum(serialize = "kebab-case")]
    KebabCase,
    #[strum(serialize = "PascalCase")]
    PascalCase,
}

impl CaseStyle {
    fn transform_text(&self, input: &str) -> String {
        match self {
            Self::KebabCase => input.to_kebab_case(),
            Self::PascalCase => input.to_pascal_case(),
        }
    }
}

#[derive(Default)]
pub struct EnumAttrs {
    pub case_style: Option<CaseStyle>,
    pub ns_strict: bool,
}

fn parse_enum_attrs(attrs: &[Attribute]) -> EnumAttrs {
    let enum_attrs = EnumAttrs::default();

    enum_attrs
}

#[derive(Default)]
pub struct StructAttrs {
    pub root: Option<LitByteStr>,
}

pub fn parse_struct_attrs(attrs: &[Attribute]) -> StructAttrs {
    let mut struct_attrs = StructAttrs::default();

    let attrs = get_scoped_attrs(attrs);
    let attrs = if let Some(attrs) = attrs {
        attrs
    } else {
        return struct_attrs;
    };

    for meta in attrs {
        match meta {
            // single flag
            Meta::Path(_path) => {
                panic!("invalid path attribute")
            }
            Meta::List(list) => {
                panic!("list-type attrs not supported: {}", list.to_token_stream());
            }
            Meta::NameValue(name_value) => {
                if name_value.path.is_ident("root") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::ByteStr(lit_str),
                        ..
                    }) = name_value.value
                    {
                        struct_attrs.root = Some(lit_str);
                    }
                } else {
                    panic!(
                        "unrecognized field attribute: {}",
                        name_value.to_token_stream()
                    );
                }
            }
        }
    }

    struct_attrs
}

#[derive(Default)]
pub struct FieldAttrs {
    pub rename: Option<String>,
    pub ns: Option<String>,
    pub text: bool,
    pub untagged: bool,
    pub flatten: bool,
    pub default: Option<syn::ExprPath>,
}

pub fn parse_field_attrs(attrs: &[Attribute]) -> FieldAttrs {
    let mut field_attrs = FieldAttrs::default();

    let attrs = get_scoped_attrs(attrs);
    let attrs = if let Some(attrs) = attrs {
        attrs
    } else {
        return field_attrs;
    };

    for meta in attrs {
        match meta {
            // single flag
            Meta::Path(path) => {
                if path.is_ident("text") {
                    field_attrs.text = true;
                }
                if path.is_ident("untagged") {
                    field_attrs.untagged = true;
                }
                if path.is_ident("flatten") {
                    field_attrs.flatten = true;
                }
            }
            Meta::List(list) => {
                panic!("list-type attrs not supported: {}", list.to_token_stream());
            }
            Meta::NameValue(name_value) => {
                if name_value.path.is_ident("ns") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = name_value.value
                    {
                        field_attrs.ns = Some(lit_str.value());
                    }
                } else if name_value.path.is_ident("rename") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = name_value.value
                    {
                        field_attrs.rename = Some(lit_str.value());
                    } else {
                        panic!("invalid rename attribute");
                    }
                } else if name_value.path.is_ident("default") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = name_value.value
                    {
                        let a: syn::ExprPath = syn::parse_str(&lit_str.value())
                            .expect("could not parse default attribute as expression");
                        field_attrs.default = Some(a);
                    } else {
                        panic!("invalid default attribute");
                    }
                } else {
                    panic!(
                        "unrecognized field attribute: {}",
                        name_value.to_token_stream()
                    );
                }
            }
        }
    }

    field_attrs
}
