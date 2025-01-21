use super::code::WgslShaderModuleSectionCode;
#[derive(Clone, Debug, PartialEq)]
/// assignments using let can happen within functions and we don't care about them, we don't need to change anything
pub struct WgslConstAssignment {
    pub code: WgslShaderModuleSectionCode,
}

impl WgslConstAssignment {
    pub fn new(name: &str, scalar_type: &str, value: &str) -> Self {
        Self {
            code: WgslShaderModuleSectionCode {
                rust_code: format!("const {}: {} = {};", name, scalar_type, value),
                wgsl_code: format!("override {}: {} = {};", name, scalar_type, value),
            },
        }
    }
    pub fn no_default(name: &str, scalar_type: &str) -> Self {
        Self {
            code: WgslShaderModuleSectionCode {
                rust_code: format!("const {}: {};", name, scalar_type),
                wgsl_code: format!("override {}: {};", name, scalar_type),
            },
        }
    }
}
