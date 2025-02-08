use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    pipeline::output::{
        generate_required_imports::generate_required_imports,
        module_for_cpu::lib::generate_module_for_cpu_usage,
        shader_module_object::generate_shader_module_object,
        types_for_rust_usage::types::define_types_for_use_in_rust_and_set_binding_numbers,
    },
    state::ModuleTransformState,
};
pub fn generate_user_facing_module(state: &mut ModuleTransformState) -> TokenStream {
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
    let types = define_types_for_use_in_rust_and_set_binding_numbers(state);
    let object = generate_shader_module_object(state);
    let module_for_cpu = generate_module_for_cpu_usage(state);
    let required_imports = generate_required_imports();
    quote!(
        #module_visibility mod #module_ident {
            #required_imports

            #types

            #object

            #module_for_cpu
        }
    )
}
