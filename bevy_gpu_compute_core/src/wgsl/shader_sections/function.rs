use super::code::WgslShaderModuleSectionCode;
#[derive(Clone, Debug, PartialEq)]

pub struct WgslFunction {
    pub name: String,
    pub code: WgslShaderModuleSectionCode,
}
