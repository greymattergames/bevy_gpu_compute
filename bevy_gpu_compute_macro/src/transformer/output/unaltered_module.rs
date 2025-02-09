use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;

use crate::state::ModuleTransformState;
pub fn generate_unaltered_module(state: &ModuleTransformState) -> TokenStream {
    let module_ident: TokenStream = if let Some(c) = &state.module_ident {
        format!("{}_for_syntax_check", c).parse().unwrap()
    } else {
        abort!(
            state.rust_module.ident.span(),
            "No module ident found in transform state"
        );
    };
    let module_visibility: TokenStream = if let Some(c) = &state.module_visibility {
        c.parse().unwrap()
    } else {
        abort!(
            state.rust_module.ident.span(),
            "No module visibility found in transform state"
        );
    };
    let content: TokenStream = state.get_original_content().parse().unwrap();
    quote!(
    #module_visibility mod #module_ident {
        use super::*;

        #content
    })
}
