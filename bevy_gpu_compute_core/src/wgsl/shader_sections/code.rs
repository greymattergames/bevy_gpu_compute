/// includes just the parts the user has input, with any relevant metadata necessary for the library to complete the module

#[derive(Debug, Clone, PartialEq)]
pub struct WgslShaderModuleSectionCode {
    pub wgsl_code: String,
}
