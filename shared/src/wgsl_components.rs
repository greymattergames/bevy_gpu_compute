use crate::custom_type_name::CustomTypeName;

/// includes just the parts the user has input, with any relevant metadata necessary for the library to complete the module

pub trait SelfToStructInitializer {
    fn to_struct_initializer(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct WgslShaderModuleComponent {
    pub rust_code: String,
    pub wgsl_code: String,
}
impl SelfToStructInitializer for WgslShaderModuleComponent {
    fn to_struct_initializer(&self) -> String {
        format!(
            "WgslShaderModuleComponent {{
                rust_code: \"{}\".to_string(),
                wgsl_code: \"{}\".to_string(),
            }}",
            self.rust_code, self.wgsl_code
        )
    }
}
#[derive(Clone)]
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
impl SelfToStructInitializer for WgslType {
    fn to_struct_initializer(&self) -> String {
        format!(
            "WgslType {{
                name: {},
                code: {},
            }}",
            self.name.to_struct_initializer(),
            self.code.to_struct_initializer()
        )
    }
}
#[derive(Debug, Clone)]
pub struct WgslDerivedType {
    pub name: String,
    pub code: WgslShaderModuleComponent,
}
impl SelfToStructInitializer for WgslDerivedType {
    fn to_struct_initializer(&self) -> String {
        format!(
            "WgslDerivedType {{
                name: \"{}\".to_string(),
                code: {},
            }}",
            self.name,
            self.code.to_struct_initializer()
        )
    }
}
#[derive(Clone)]

pub struct WgslFunction {
    pub name: String,
    pub code: WgslShaderModuleComponent,
}
impl SelfToStructInitializer for WgslFunction {
    fn to_struct_initializer(&self) -> String {
        format!(
            "WgslFunction {{
                name: \"{}\".to_string(),
                code: {},
            }}",
            self.name,
            self.code.to_struct_initializer()
        )
    }
}

#[derive(Clone)]
/// assignments using let can happen within functions and we don't care about them, we don't need to change anything
pub struct WgslConstAssignment {
    pub code: WgslShaderModuleComponent,
}
impl SelfToStructInitializer for WgslConstAssignment {
    fn to_struct_initializer(&self) -> String {
        format!(
            "WgslConstAssignment {{
                code: {},
            }}",
            self.code.to_struct_initializer()
        )
    }
}
#[derive(Clone)]

pub struct WgslArrayLength {
    pub name: String,
    pub code: WgslShaderModuleComponent,
}
impl SelfToStructInitializer for WgslArrayLength {
    fn to_struct_initializer(&self) -> String {
        format!(
            "WgslArrayLength {{
                name: \"{}\".to_string(),
                code: {},
            }}",
            self.name,
            self.code.to_struct_initializer()
        )
    }
}
#[derive(Clone)]

pub struct WgslInputArray {
    pub item_type: WgslType,
    pub array_type: WgslDerivedType,
}
impl SelfToStructInitializer for WgslInputArray {
    fn to_struct_initializer(&self) -> String {
        format!(
            "WgslInputArray {{
                item_type: {},
                array_type: {},
            }}",
            self.item_type.to_struct_initializer(),
            self.array_type.to_struct_initializer()
        )
    }
}
#[derive(Clone)]

pub struct WgslOutputArray {
    pub item_type: WgslType,
    pub array_type: WgslDerivedType,
    pub atomic_counter_type: Option<WgslDerivedType>,
}
impl SelfToStructInitializer for WgslOutputArray {
    fn to_struct_initializer(&self) -> String {
        format!(
            "WgslOutputArray {{
                item_type: {},
                array_type: {},
                atomic_counter_type: {},
            }}",
            self.item_type.to_struct_initializer(),
            self.array_type.to_struct_initializer(),
            self.atomic_counter_type
                .as_ref()
                .map_or("None".to_string(), |counter| {
                    counter.to_struct_initializer()
                })
        )
    }
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
pub struct WgslWorkgroupDeclaration {
    pub shader_type: WgpuShaderType,
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
impl ToString for WgslWorkgroupDeclaration {
    fn to_string(&self) -> String {
        return format!(
            "@{} @workgroup_size({}, {}, {})\n",
            self.shader_type.to_string(),
            self.x,
            self.y,
            self.z
        );
    }
}
