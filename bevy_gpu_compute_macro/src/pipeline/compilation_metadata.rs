use crate::pipeline::phases::custom_type_collector::custom_type::CustomType;
use bevy_gpu_compute_core::wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion;
use proc_macro2::TokenStream;

pub struct CompilationMetadata {
    pub main_func_required: bool,
    pub custom_types: Option<Vec<CustomType>>,
    pub wgsl_module_user_portion: Option<WgslShaderModuleUserPortion>,
    pub typesafe_buffer_builders: Option<TokenStream>,
}
