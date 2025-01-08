use proc_macro_error::abort;
use quote::ToTokens;
use syn::{parse_quote, spanned::Spanned, visit_mut::VisitMut};

use crate::transformer::custom_types::custom_type::CustomType;

pub struct ArrayToWgslTransformer {}

impl VisitMut for ArrayToWgslTransformer {
    fn visit_type_mut(&mut self, t: &mut syn::Type) {
        syn::visit_mut::visit_type_mut(self, t);
        if let syn::Type::Array(arr) = t {
            let arr = array_to_wgsl(arr);
            *t = arr;
        }
    }
}

pub fn array_to_wgsl(arr: &syn::TypeArray) -> syn::Type {
    let t = &arr.elem;
    let len = format_array_len(&arr.len);
    return parse_quote!(array<#t,#len>);
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
