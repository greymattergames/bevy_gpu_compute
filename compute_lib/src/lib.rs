// Re-export the proc macros
extern crate proc_macro_lib;

// use my_derive::MyMacroHere;
pub use proc_macro_lib::{ComputeInput, ComputeOutput, compute_shader};

// Traits with proper bounds
pub trait ComputeInput: Sized {
    type Inner: Sized;
    fn as_slice(&self) -> &[Self::Inner];
    fn from_slice(slice: &[Self::Inner]) -> Vec<Self>;
}

pub trait ComputeOutput: Sized {
    fn as_slice(&self) -> &[Self];
    fn from_slice(slice: &[Self]) -> Vec<Self>;
}

pub trait WGSLType {
    const TYPE_NAME: &'static str;
    const STORAGE_COMPATIBLE: bool = true;
}

// Vec types with WGSLType implementations
#[derive(Clone, Copy)]
pub struct Vec2<T>(pub T, pub T);

impl<T: WGSLType> WGSLType for Vec2<T> {
    const TYPE_NAME: &'static str = "vec2<T>";
}

#[derive(Clone, Copy)]
pub struct Vec3<T>(pub T, pub T, pub T);

impl<T: WGSLType> WGSLType for Vec3<T> {
    const TYPE_NAME: &'static str = "vec3<T>";
}

// Implementations for primitive types
impl WGSLType for f32 {
    const TYPE_NAME: &'static str = "f32";
}

impl WGSLType for u32 {
    const TYPE_NAME: &'static str = "u32";
}

// Global ID function
pub fn global_id() -> Vec3<u32> {
    unimplemented!("This function should only be called in compute shaders")
}

// Add WGSLShader trait
pub trait WGSLShader {
    fn wgsl_code() -> String;
    fn debug_wgsl() -> String {
        format!(
            "// Generated WGSL for shader: {}\n\n{}",
            std::any::type_name::<Self>(),
            Self::wgsl_code()
        )
    }
}
