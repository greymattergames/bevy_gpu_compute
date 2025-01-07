use proc_macro_error::abort;
use quote::ToTokens;
use syn::spanned::Spanned;

use crate::transformer::custom_types::custom_type::CustomType;

use super::r#type::type_to_wgsl;

pub fn array_to_wgsl(arr: &syn::TypeArray, custom_types: &Vec<CustomType>) -> String {
    let item_type = type_to_wgsl(&arr.elem, custom_types);
    return format!("array<{},{}>", item_type, format_array_len(&arr.len));
}

fn format_array_len(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(int) => int.to_string(),
            _ => abort!(lit.span(), "Array length must be an integer literal"),
        },
        _ => abort!(expr.span(), "Array length must be a constant expression"),
    }
}
