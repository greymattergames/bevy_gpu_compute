use super::super::shader_custom_type_name::ShaderCustomTypeName;
use super::code::WgslShaderModuleSectionCode;
#[derive(Debug, Clone, PartialEq)]
pub struct WgslType {
    pub name: ShaderCustomTypeName,
    pub code: WgslShaderModuleSectionCode,
}
