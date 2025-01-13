use std::alloc::Global;

use shared::wgsl_components::{
    WgslConstAssignment, WgslFunction, WgslOutputArray, WgslShaderModuleUserPortion, WgslType,
};
use syn::{Item, ItemMod, token::Brace};

use crate::transformer::allowed_types::AllowedRustTypes;

pub struct ModuleTransformState {
    _original_content: String,
    pub rust_module: ItemMod,
    pub allowed_types: Option<AllowedRustTypes>,
    pub module_visibility: Option<String>,
    pub module_ident: Option<String>,
    pub result: WgslShaderModuleUserPortion,
}
impl ModuleTransformState {
    pub fn empty(rust_module: ItemMod, content: String) -> Self {
        Self {
            _original_content: content,
            rust_module,
            allowed_types: None,
            module_visibility: None,
            module_ident: None,
            result: WgslShaderModuleUserPortion::empty(),
        }
    }
    pub fn get_original_content(&self) -> String {
        self._original_content.clone()
    }
}
