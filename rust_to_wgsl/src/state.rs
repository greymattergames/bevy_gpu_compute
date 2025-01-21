use shared::wgsl_components::WgslShaderModuleUserPortion;
use syn::ItemMod;

use crate::transformer::custom_types::custom_type::CustomType;

pub struct ModuleTransformState {
    _original_content: String,
    pub rust_module: ItemMod,
    pub custom_types: Option<Vec<CustomType>>,
    pub module_visibility: Option<String>,
    pub module_ident: Option<String>,
    pub result: WgslShaderModuleUserPortion,
}
impl ModuleTransformState {
    pub fn empty(rust_module: ItemMod, content: String) -> Self {
        Self {
            _original_content: content,
            rust_module,
            custom_types: None,
            module_visibility: None,
            module_ident: None,
            result: WgslShaderModuleUserPortion::empty(),
        }
    }
    pub fn get_original_content(&self) -> String {
        self._original_content.clone()
    }
}
