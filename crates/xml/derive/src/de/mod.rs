pub mod attrs;
mod de_enum;
mod de_struct;
mod field;

pub use de_enum::Enum;
pub use de_struct::NamedStruct;
pub use field::Field;

pub fn get_generic_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                args,
                ..
            }) = &seg.arguments
            {
                if let Some(syn::GenericArgument::Type(t)) = &args.first() {
                    return Some(t);
                }
            }
        }
    }
    None
}
