use crate::custom_type_name::CustomTypeName;

/// includes just the parts the user has input, with any relevant metadata necessary for the library to complete the module

#[derive(Debug, Clone)]
pub struct WgslShaderModuleComponent {
    pub rust_code: String,
    pub wgsl_code: String,
}

#[derive(Clone, Debug)]
pub struct WgslShaderModuleUserPortion {
    /// defined with the "const" keyword
    /// single line
    /// value remains static
    /// type must be wgsl type or created somewhere else in the module
    /// value could be a type instantiation, a scalar, or a function
    pub static_consts: Vec<WgslConstAssignment>,
    /// defined with either struct keyword, or a type alias
    /// These are not associated with any buffers and exist only on the GPU
    pub helper_types: Vec<WgslType>,
    /// identified with a #[config_input] attribute above them
    pub uniforms: Vec<WgslType>,
    /// identified with a #[vec_input] attribute above them
    pub input_arrays: Vec<WgslInputArray>,
    /// identified with a #[vec_output] attribute above them
    pub output_arrays: Vec<WgslOutputArray>,
    /// any function that appears besides the one called "main"
    pub helper_functions: Vec<WgslFunction>,
    /// the main function, identified by its name: "main"
    /// MUST contain a single parameter called "global_id" of type "WgslGlobalId"
    /// look for any attempt to ASSIGN to the value of "global_id.x", "global_id.y", or "global_id.z" or just "global_id" and throw an error
    pub main_function: Option<WgslFunction>,
}
impl WgslShaderModuleUserPortion {
    pub fn empty() -> Self {
        Self {
            static_consts: vec![],
            helper_types: vec![],
            uniforms: vec![],
            input_arrays: vec![],
            output_arrays: vec![],
            helper_functions: vec![],
            main_function: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WgslType {
    pub name: CustomTypeName,
    pub code: WgslShaderModuleComponent,
}

#[derive(Debug, Clone)]
pub struct WgslDerivedType {
    pub name: String,
    pub code: WgslShaderModuleComponent,
}

#[derive(Clone, Debug)]

pub struct WgslFunction {
    pub name: String,
    pub code: WgslShaderModuleComponent,
}

#[derive(Clone, Debug)]
/// assignments using let can happen within functions and we don't care about them, we don't need to change anything
pub struct WgslConstAssignment {
    pub code: WgslShaderModuleComponent,
}

impl WgslConstAssignment {
    pub fn new(name: &str, scalar_type: &str, value: &str) -> Self {
        Self {
            code: WgslShaderModuleComponent {
                rust_code: format!("const {}: {} = {};", name, scalar_type, value),
                wgsl_code: format!("override {}: {} = {};", name, scalar_type, value),
            },
        }
    }
}
#[derive(Clone, Debug)]

pub struct WgslArrayLength {
    pub name: String,
    pub code: WgslShaderModuleComponent,
}

#[derive(Clone, Debug)]

pub struct WgslInputArray {
    pub item_type: WgslType,
    pub array_type: WgslDerivedType,
}

#[derive(Clone, Debug)]

pub struct WgslOutputArray {
    pub item_type: WgslType,
    pub array_type: WgslDerivedType,
    pub atomic_counter_name: Option<String>,
}

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
