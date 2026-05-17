use syn::{GenericArgument, PathArguments, Type};

pub fn is_known_binary_type(ty: &Type) -> bool {
    match ty {
        Type::Path(p) => {
            let Some(segment) = p.path.segments.last() else { return false; };
            let ident = segment.ident.to_string();
            if ident == "Vec" || ident == "VecDeque" || ident == "LinkedList" || ident == "Box" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return is_known_binary_type(inner_ty);
                    }
                }
            }
            ident == "Uint8Array" || ident == "u8"
        }
        Type::Reference(r) => is_known_binary_type(&r.elem),
        Type::Slice(s) => is_known_binary_type(&s.elem),
        _ => false,
    }
}
