use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    state::ModuleTransformState,
    transformer::output::{
        shader_module_object::generate_shader_module_object,
        types_for_rust_usage::types::define_types_for_use_in_rust,
    },
};
pub fn generate_expanded_module(state: &ModuleTransformState) -> TokenStream {
    let module_ident: TokenStream = if let Some(c) = &state.module_ident {
        c.parse().unwrap()
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
    let types = define_types_for_use_in_rust(state);
    let object = generate_shader_module_object(state);
    quote!(
        #module_visibility mod #module_ident {
            use shared::wgsl_components::*; //todo, make this less brittle
            use shared::custom_type_name::*;
            use shared::misc_types::*;
            use shared::wgsl_in_rust_helpers::*;
            // use shared::wgsl_in_rust_helpers::pod_bool::PodBool;

            #types

            #object
        }
    )
}
