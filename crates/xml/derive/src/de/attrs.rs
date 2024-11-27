use darling::{util::Flag, FromDeriveInput, FromField, FromMeta, FromVariant};
use syn::LitByteStr;

#[derive(Default, FromMeta)]
pub struct ContainerAttrs {
    pub ns_strict: Flag,
}

#[derive(Default, FromMeta)]
pub struct TagAttrs {
    pub rename: Option<LitByteStr>,
    pub ns_strict: Flag,
    pub ns: Option<LitByteStr>,
}

#[derive(Default, FromVariant)]
#[darling(attributes(xml))]
pub struct VariantAttrs {
    #[darling(flatten)]
    pub common: TagAttrs,
}

#[derive(Default, FromDeriveInput)]
#[darling(attributes(xml))]
pub struct EnumAttrs {
    #[darling(flatten)]
    container: ContainerAttrs,
}

#[derive(Default, FromDeriveInput)]
#[darling(attributes(xml))]
pub struct StructAttrs {
    #[darling(flatten)]
    pub container: ContainerAttrs,

    pub root: Option<LitByteStr>,
}

#[derive(Default, FromField)]
#[darling(attributes(xml))]
pub struct FieldAttrs {
    #[darling(flatten)]
    pub common: TagAttrs,
    pub text: Flag,
    pub untagged: Flag,
    pub flatten: Flag,
    pub default: Option<syn::ExprPath>,
}
