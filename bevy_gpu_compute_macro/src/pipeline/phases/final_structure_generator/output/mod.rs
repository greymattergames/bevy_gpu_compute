use crate::state::ModuleTransformState;
use proc_macro2::TokenStream;
use quote::quote;
use unaltered_module::generate_unaltered_module;
use user_facing_module::generate_expanded_module;

mod generate_required_imports;
mod module_for_cpu;
mod per_component_expansion;
mod shader_module_object;
mod types_for_rust_usage;
mod unaltered_module;
mod user_facing_module;
pub fn produce_expanded_output(state: &mut ModuleTransformState) -> TokenStream {
    let unaltered_module = generate_unaltered_module(state);
    let expanded_module = generate_expanded_module(state);
    quote!(
        #unaltered_module

        #expanded_module
    )
}
