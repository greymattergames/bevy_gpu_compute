use super::custom_types::custom_type::{CustomType, CustomTypeKind};

const WGSL_NATIVE_TYPES: [&str; 11] = [
    "f32", "f16", "i32", "u32", "bool", "Vec2", "Vec3", "Vec4", "Mat2x2", "Mat3x3", "Mat4x4",
];
const LIB_HELPER_TYPES: [&str; 5] = [
    "WgslScalar",
    "WgslGlobalId",
    "WgslConfigInput",
    "WgslVecInput",
    "WgslOutput",
];

pub struct AllowedRustTypes {
    pub wgsl_native_types: Vec<String>,
    pub lib_helper_types: Vec<String>,
    pub user_declared_types: Vec<CustomType>,
}

impl AllowedRustTypes {
    pub fn new(user_declared_types: Vec<CustomType>) -> Self {
        AllowedRustTypes {
            wgsl_native_types: WGSL_NATIVE_TYPES.iter().map(|s| s.to_string()).collect(),
            lib_helper_types: LIB_HELPER_TYPES.iter().map(|s| s.to_string()).collect(),
            user_declared_types,
        }
    }
    pub fn add_user_type(&mut self, name: String, kind: CustomTypeKind) {
        todo!()
        // self.user_declared_types.push(CustomType { name, kind });
    }
}
