use darling::{util::Flag, FromDeriveInput, FromField, FromMeta, FromVariant};
use syn::LitByteStr;

#[derive(Default, FromMeta, Clone)]
pub struct ContainerAttrs {
    pub ns_strict: Flag,
}

#[derive(Default, FromMeta, Clone)]
pub struct TagAttrs {
    pub rename: Option<LitByteStr>,
    pub ns_strict: Flag,
    pub ns: Option<LitByteStr>,
}

#[derive(Default, FromVariant, Clone)]
#[darling(attributes(xml))]
pub struct VariantAttrs {
    #[darling(flatten)]
    pub common: TagAttrs,
}

#[derive(Default, FromDeriveInput, Clone)]
#[darling(attributes(xml))]
pub struct EnumAttrs {
    #[darling(flatten)]
    container: ContainerAttrs,
}

#[derive(Default, FromDeriveInput, Clone)]
#[darling(attributes(xml))]
pub struct StructAttrs {
    #[darling(flatten)]
    pub container: ContainerAttrs,

    pub root: Option<LitByteStr>,
    pub allow_invalid: Flag,
}

#[derive(Default, FromMeta, PartialEq)]
pub enum FieldType {
    #[default]
    Tag,
    Attr,
    Text,
    Untagged,
}

#[derive(Default, FromField)]
#[darling(attributes(xml))]
pub struct FieldAttrs {
    #[darling(flatten)]
    pub common: TagAttrs,
    pub flatten: Flag,
    pub default: Option<syn::ExprPath>,
    #[darling(default, rename = "ty")]
    pub xml_ty: FieldType,
}
