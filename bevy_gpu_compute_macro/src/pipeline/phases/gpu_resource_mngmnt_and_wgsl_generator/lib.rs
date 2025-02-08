use bevy_gpu_compute_core::wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion;
use proc_macro_error::abort;
use quote::ToTokens;

use crate::pipeline::phases::custom_type_collector::custom_type::CustomType;

use super::constants::extract_constants;
use super::divide_custom_types::generate_helper_types_inputs_and_outputs_for_wgsl_module_def;
use super::helper_functions::extract_helper_functions;
use super::main_function::parse_main_function;

/// This will also change custom_types
pub fn parse_shader_module_for_gpu(
    rust_module_transformed_for_gpu: &syn::ItemMod,
    custom_types: &Vec<CustomType>,
) -> (WgslShaderModuleUserPortion, Vec<CustomType>) {
    let mut out_module: WgslShaderModuleUserPortion = WgslShaderModuleUserPortion::empty();
    out_module.main_function = Some(parse_main_function(
        rust_module_transformed_for_gpu,
        custom_types,
    ));
    out_module.static_consts = extract_constants(rust_module_transformed_for_gpu, custom_types);
    out_module.helper_functions =
        extract_helper_functions(rust_module_transformed_for_gpu, custom_types);
    let new_custom_types =
        generate_helper_types_inputs_and_outputs_for_wgsl_module_def(custom_types, &mut out_module);
    (out_module, new_custom_types)
}
