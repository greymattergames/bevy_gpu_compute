pub enum WgpuShaderType {
    Compute,
    Vertex,
    Fragment,
}
impl ToString for WgpuShaderType {
    fn to_string(&self) -> String {
        match self {
            WgpuShaderType::Compute => "compute".to_string(),
            WgpuShaderType::Vertex => panic!("Vertex shaders not yet supported"),
            WgpuShaderType::Fragment => panic!("Fragment shaders not yet supported"),
        }
    }
}

pub const WORKGROUP_SIZE_X_VAR_NAME: &str = "_LIB_WORKGROUP_SIZE_X";
pub const WORKGROUP_SIZE_Y_VAR_NAME: &str = "_LIB_WORKGROUP_SIZE_Y";
pub const WORKGROUP_SIZE_Z_VAR_NAME: &str = "_LIB_WORKGROUP_SIZE_Z";
pub struct WgslWorkgroupDeclaration {
    pub shader_type: WgpuShaderType,
}
impl ToString for WgslWorkgroupDeclaration {
    fn to_string(&self) -> String {
        return format!(
            "@{} @workgroup_size({}, {}, {})\n",
            self.shader_type.to_string(),
            WORKGROUP_SIZE_X_VAR_NAME,
            WORKGROUP_SIZE_Y_VAR_NAME,
            WORKGROUP_SIZE_Z_VAR_NAME
        );
    }
}