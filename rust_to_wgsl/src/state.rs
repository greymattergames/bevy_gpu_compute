use shared::wgsl_components::{
    WgslConstAssignment, WgslFunction, WgslOutputArray, WgslShaderModuleUserPortion, WgslType,
};
use syn::ItemMod;

use crate::transformer::allowed_types::AllowedRustTypes;

pub struct ModuleTransformState {
    pub rust_module: ItemMod,
    pub allowed_types: Option<AllowedRustTypes>,
    pub module_visibility: Option<String>,
    pub module_ident: Option<String>,
    pub result: WgslShaderModuleUserPortion,
}
impl ModuleTransformState {
    pub fn empty(rust_module: ItemMod) -> Self {
        Self {
            rust_module,
            allowed_types: None,
            module_visibility: None,
            module_ident: None,
            result: WgslShaderModuleUserPortion::empty(),
        }
    }
}
