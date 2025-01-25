use proc_macro_error::abort;
use syn::{parse_quote, spanned::Spanned, visit_mut::VisitMut};

pub struct ArrayToWgslTransformer {}

impl VisitMut for ArrayToWgslTransformer {
    fn visit_item_type_mut(&mut self, t: &mut syn::ItemType) {
        syn::visit_mut::visit_item_type_mut(self, t);
        if let syn::Type::Array(arr) = *t.ty.clone() {
            let type_path = array_to_wgsl(&arr);
            *t.ty = syn::Type::Path(type_path);
        }
    }
    fn visit_pat_type_mut(&mut self, t: &mut syn::PatType) {
        syn::visit_mut::visit_pat_type_mut(self, t);
        if let syn::Type::Array(arr) = *t.ty.clone() {
            let type_path = array_to_wgsl(&arr);
            *t.ty = syn::Type::Path(type_path);
        }
    }
}

pub fn array_to_wgsl(arr: &syn::TypeArray) -> syn::TypePath {
    let ident = match *arr.elem.clone() {
        syn::Type::Path(p) => {
            if let Some(f) = p.path.segments.first() {
                f.ident.clone()
            } else {
                abort!(arr.elem.span(), "Array element type is not a path")
            }
        }
        _ => abort!(arr.elem.span(), "Array element type is not a path"),
    };
    let len = arr.len.clone();

    parse_quote!(array<#ident,#len>)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::{TypeArray, parse_quote};

    #[test]
    fn test_array_to_wgsl() {
        let input: TypeArray = parse_quote! { [f32; 4] };
        let output = array_to_wgsl(&input);
        assert_eq!(output.to_token_stream().to_string(), "array < f32 , 4 >");
    }
}
