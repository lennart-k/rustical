use std::collections::HashMap;

use darling::{util::Flag, FromDeriveInput, FromField, FromMeta, FromVariant};
use syn::LitByteStr;

#[derive(Default, FromMeta, Clone)]
pub struct TagAttrs {
    pub rename: Option<LitByteStr>,
    pub ns: Option<syn::Path>,
}

#[derive(Default, FromVariant)]
#[darling(attributes(xml))]
pub struct VariantAttrs {
    #[darling(flatten)]
    pub common: TagAttrs,
    pub other: Flag,
    pub skip_deserializing: Flag,
}

#[derive(Default, FromDeriveInput, Clone)]
#[darling(attributes(xml))]
pub struct EnumAttrs {
    pub untagged: Flag,
    pub unit_variants_name: Option<String>,
}

#[derive(Default, FromDeriveInput, Clone)]
#[darling(attributes(xml))]
pub struct StructAttrs {
    pub root: Option<LitByteStr>,
    pub ns: Option<syn::Path>,
    #[darling(default)]
    pub ns_prefix: HashMap<syn::Path, LitByteStr>,
    pub allow_invalid: Flag,
}

#[derive(Default, FromMeta, PartialEq)]
pub enum FieldType {
    #[default]
    Tag,
    Attr,
    Text,
    Untagged,
    TagName,
    Namespace,
}

#[derive(Default, FromField)]
#[darling(attributes(xml))]
pub struct FieldAttrs {
    #[darling(flatten)]
    pub common: TagAttrs,
    pub flatten: Flag,
    pub default: Option<syn::ExprPath>,
    pub serialize_with: Option<syn::ExprPath>,
    #[darling(default, rename = "ty")]
    pub xml_ty: FieldType,
}
