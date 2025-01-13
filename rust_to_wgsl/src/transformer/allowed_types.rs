use super::custom_types::custom_type::{CustomType, CustomTypeKind};

pub const WGSL_NATIVE_TYPES: [&str; 65] = [
    "Vec2I32",
    "Vec2U32",
    "Vec2F32",
    "Vec2F16",
    "Vec2Bool",
    "Vec3I32",
    "Vec3U32",
    "Vec3F32",
    "Vec3F16",
    "Vec3Bool",
    "Vec4I32",
    "Vec4U32",
    "Vec4F32",
    "Vec4F16",
    "Vec4Bool",
    "Mat2x2I32",
    "Mat2x2U32",
    "Mat2x2F32",
    "Mat2x2F16",
    "Mat2x2Bool",
    "Mat2x3U32",
    "Mat2x3I32",
    "Mat2x3F32",
    "Mat2x3F16",
    "Mat2x3Bool",
    "Mat2x4I32",
    "Mat2x4U32",
    "Mat2x4F32",
    "Mat2x4F16",
    "Mat2x4Bool",
    "Mat3x2I32",
    "Mat3x2U32",
    "Mat3x2F32",
    "Mat3x2F16",
    "Mat3x2Bool",
    "Mat3x3I32",
    "Mat3x3U32",
    "Mat3x3F32",
    "Mat3x3F16",
    "Mat3x3Bool",
    "Mat3x4I32",
    "Mat3x4U32",
    "Mat3x4F32",
    "Mat3x4F16",
    "Mat3x4Bool",
    "Mat4x2I32",
    "Mat4x2U32",
    "Mat4x2F32",
    "Mat4x2F16",
    "Mat4x2Bool",
    "Mat4x3I32",
    "Mat4x3U32",
    "Mat4x3F32",
    "Mat4x3F16",
    "Mat4x3Bool",
    "Mat4x4I32",
    "Mat4x4U32",
    "Mat4x4F32",
    "Mat4x4F16",
    "Mat4x4Bool",
    "f32",
    "f16",
    "i32",
    "u32",
    "bool",
];
const LIB_HELPER_TYPES: [&str; 5] = [
    "WgslScalar",
    "WgslGlobalId",
    "WgslConfigInput",
    "WgslVecInput",
    "WgslOutput",
];

#[derive(Debug, Clone)]
pub struct AllowedRustTypes {
    pub wgsl_native_types: Vec<String>,
    pub lib_helper_types: Vec<String>,
    pub custom_types: Vec<CustomType>,
}

impl AllowedRustTypes {
    pub fn new(custom_types: Vec<CustomType>) -> Self {
        AllowedRustTypes {
            wgsl_native_types: WGSL_NATIVE_TYPES.iter().map(|s| s.to_string()).collect(),
            lib_helper_types: LIB_HELPER_TYPES.iter().map(|s| s.to_string()).collect(),
            custom_types,
        }
    }
    pub fn add_user_type(&mut self, custom_type: CustomType) {
        self.custom_types.push(custom_type);
    }
}
