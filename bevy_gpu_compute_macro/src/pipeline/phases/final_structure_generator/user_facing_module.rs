use bevy_gpu_compute_core::wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion;
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{ItemMod, parse_quote};

use crate::pipeline::phases::{
    custom_type_collector::custom_type::CustomType,
    final_structure_generator::generate_required_imports::generate_required_imports,
};

use super::{
    shader_module_object::generate_shader_module_object,
    types_for_rust_usage::define_types_for_use_in_rust_and_set_binding_numbers,
};

pub fn generate_user_facing_module(
    custom_types: &Vec<CustomType>,
    wgsl_shader_module: &mut WgslShaderModuleUserPortion,
    rust_module_for_cpu: &ItemMod,
    builders: &TokenStream,
) -> TokenStream {
    let mut module_to_add_to = rust_module_for_cpu.clone();
    let generated_types =
        define_types_for_use_in_rust_and_set_binding_numbers(custom_types, wgsl_shader_module);
    let generated_shader_module_object = generate_shader_module_object(wgsl_shader_module);
    let required_imports = generate_required_imports();
    module_to_add_to
        .content
        .as_mut()
        .unwrap()
        .1
        /* this may produce an error, if things aren't working, restructure this... since it may not be able to parse all these items into a single syn::Item */
        .push(parse_quote! (
                // #required_imports

                // #generated_types

                // #generated_shader_module_object

                // #builders
        ));
    module_to_add_to.into_token_stream()
}
