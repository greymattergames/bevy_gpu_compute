use bevy_gpu_compute_core::wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::ItemMod;

use crate::pipeline::phases::final_structure_generator::generate_required_imports::generate_required_imports;

use super::{
    shader_module_object::generate_shader_module_object,
    types_for_rust_usage::define_types_for_use_in_rust_and_set_binding_numbers,
};

pub fn generate_user_facing_module(
    wgsl_shader_module: &mut WgslShaderModuleUserPortion,
    rust_module_for_cpu: &ItemMod,
    builders: &TokenStream,
) -> TokenStream {
    let generated_types = define_types_for_use_in_rust_and_set_binding_numbers(wgsl_shader_module);
    let generated_shader_module_object = generate_shader_module_object(wgsl_shader_module);
    let required_imports = generate_required_imports();
    let user_module_content: TokenStream = rust_module_for_cpu
        .content
        .as_ref()
        .unwrap()
        .1
        .iter()
        .map(|item| item.to_token_stream())
        .collect();
    let vis = &rust_module_for_cpu.vis;
    let ident = &rust_module_for_cpu.ident;
    /* this may produce an error, if things aren't working, restructure this... since it may not be able to parse all these items into a single syn::Item */
    quote! {
        #[allow(dead_code, unused_variables, unused_imports)]
        #vis mod #ident {
            #required_imports

            #user_module_content

            #generated_types

            #generated_shader_module_object

            #builders
        }
    }
}
