pub(crate) fn get_generic_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty
        && let Some(seg) = path.segments.last()
        && let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            args, ..
        }) = &seg.arguments
        && let Some(syn::GenericArgument::Type(t)) = &args.first()
    {
        return Some(t);
    }
    None
}
