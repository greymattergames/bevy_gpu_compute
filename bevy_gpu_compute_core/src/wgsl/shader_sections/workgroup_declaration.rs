pub enum WgpuShaderType {
    Compute,
    Vertex,
    Fragment,
}
impl std::fmt::Display for WgpuShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WgpuShaderType::Compute => write!(f, "compute"),
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
impl std::fmt::Display for WgslWorkgroupDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "@{} @workgroup_size({}, {}, {})",
            self.shader_type,
            WORKGROUP_SIZE_X_VAR_NAME,
            WORKGROUP_SIZE_Y_VAR_NAME,
            WORKGROUP_SIZE_Z_VAR_NAME
        )
    }
}
