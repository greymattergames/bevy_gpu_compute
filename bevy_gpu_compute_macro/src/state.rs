use bevy_gpu_compute_core::wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion;
use syn::ItemMod;

use crate::transformer::custom_types::custom_type::CustomType;

pub struct ModuleTransformState {
    _original_content: String,
    pub rust_module: ItemMod,
    pub rust_module_for_cpu: ItemMod,
    pub custom_types: Option<Vec<CustomType>>,
    pub module_visibility: Option<String>,
    pub module_ident: Option<String>,
    pub result: WgslShaderModuleUserPortion,
    pub result_for_cpu: WgslShaderModuleUserPortion,
}
impl ModuleTransformState {
    pub fn empty(rust_module: ItemMod, content: String) -> Self {
        Self {
            _original_content: content,
            rust_module: rust_module.clone(),
            rust_module_for_cpu: rust_module.clone(),
            custom_types: None,
            module_visibility: None,
            module_ident: None,
            result: WgslShaderModuleUserPortion::empty(),
            result_for_cpu: WgslShaderModuleUserPortion::empty(),
        }
    }
    pub fn get_original_content(&self) -> String {
        self._original_content.clone()
    }
}
