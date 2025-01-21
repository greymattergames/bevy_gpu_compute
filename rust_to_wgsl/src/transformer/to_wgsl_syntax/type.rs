use proc_macro_error::abort;
use syn::{PathSegment, parse_quote, visit_mut::VisitMut};

use crate::transformer::custom_types::custom_type::CustomType;

pub struct TypeToWgslTransformer<'a> {
    pub custom_types: &'a Vec<CustomType>,
}
impl<'a> VisitMut for TypeToWgslTransformer<'a> {
    fn visit_type_path_mut(&mut self, mut t: &mut syn::TypePath) {
        syn::visit_mut::visit_type_path_mut(self, t);
        path_type_to_wgsl(&mut t, &self.custom_types);
    }
}

pub fn path_type_to_wgsl<'a>(type_path: &mut syn::TypePath, custom_types: &Vec<CustomType>) {
    let path = &mut type_path.path;
    let segments = &mut path.segments;
    for segment in segments.iter_mut() {
        let new_segment = convert_path_segment(segment.clone(), custom_types);
        *segment = new_segment;
    }
}
fn convert_path_segment(segment: PathSegment, custom_types: &Vec<CustomType>) -> PathSegment {
    let ident = &segment.ident;
    let custom_t = custom_types.iter().find(|t| t.name.eq(&ident));
    if let Some(_) = custom_t {
        segment.clone()
    } else {
        match ident.to_string().as_str() {
            "atomic" => segment.clone(),
            "array" => segment.clone(),
            "f32" => segment.clone(),
            "i32" => segment.clone(),
            "u32" => segment.clone(),
            "PodF16" => parse_quote!(f16),
            "f16" => {
                abort!(
                    ident.span(),
                    "Standard rust f16s are not \"Pods\", use `PodF16` instead of `f16`. This is because we use `bytemuck` for creating and reading GPU buffers."
                )
            }
            "bool" => segment.clone(),
            "vec3" => segment.clone(),
            "vec2" => segment.clone(),
            "vec4" => segment.clone(),
            "mat2x2" => segment.clone(),
            "mat3x3" => segment.clone(),
            "mat4x4" => segment.clone(),
            "mat2x3" => segment.clone(),
            "mat2x4" => segment.clone(),
            "mat3x2" => segment.clone(),
            "mat3x4" => segment.clone(),
            "mat4x2" => segment.clone(),
            "mat4x3" => segment.clone(),
            "WgslIterationPosition" => segment.clone(),
            "Vec2I32" => parse_quote!(vec2<i32>),
            "Vec2U32" => parse_quote!(vec2<u32>),
            "Vec2F32" => parse_quote!(vec2<f32>),
            "Vec2F16" => parse_quote!(vec2<f16>),
            "Vec2Bool" => parse_quote!(vec2<bool>),
            "Vec3I32" => parse_quote!(vec3<i32>),
            "Vec3U32" => parse_quote!(vec3<u32>),
            "Vec3F32" => parse_quote!(vec3<f32>),
            "Vec3F16" => parse_quote!(vec3<f16>),
            "Vec3Bool" => parse_quote!(vec3<bool>),
            "Vec4I32" => parse_quote!(vec4<i32>),
            "Vec4U32" => parse_quote!(vec4<u32>),
            "Vec4F32" => parse_quote!(vec4<f32>),
            "Vec4F16" => parse_quote!(vec4<f16>),
            "Vec4Bool" => parse_quote!(vec4<bool>),
            "Mat2x2I32" => parse_quote!(mat2x2<i32>),
            "Mat2x2U32" => parse_quote!(mat2x2<u32>),
            "Mat2x2F32" => parse_quote!(mat2x2<f32>),
            "Mat2x2F16" => parse_quote!(mat2x2<f16>),
            "Mat2x2Bool" => parse_quote!(mat2x2<bool>),
            "Mat2x3I32" => parse_quote!(mat2x3<i32>),
            "Mat2x3U32" => parse_quote!(mat2x3<u32>),
            "Mat2x3F32" => parse_quote!(mat2x3<f32>),
            "Mat2x3F16" => parse_quote!(mat2x3<f16>),
            "Mat2x3Bool" => parse_quote!(mat2x3<bool>),
            "Mat2x4I32" => parse_quote!(mat2x4<i32>),
            "Mat2x4U32" => parse_quote!(mat2x4<u32>),
            "Mat2x4F32" => parse_quote!(mat2x4<f32>),
            "Mat2x4F16" => parse_quote!(mat2x4<f16>),
            "Mat2x4Bool" => parse_quote!(mat2x4<bool>),
            "Mat3x2I32" => parse_quote!(mat3x2<i32>),
            "Mat3x2U32" => parse_quote!(mat3x2<u32>),
            "Mat3x2F32" => parse_quote!(mat3x2<f32>),
            "Mat3x2F16" => parse_quote!(mat3x2<f16>),
            "Mat3x2Bool" => parse_quote!(mat3x2<bool>),
            "Mat3x3I32" => parse_quote!(mat3x3<i32>),
            "Mat3x3U32" => parse_quote!(mat3x3<u32>),
            "Mat3x3F32" => parse_quote!(mat3x3<f32>),
            "Mat3x3F16" => parse_quote!(mat3x3<f16>),
            "Mat3x3Bool" => parse_quote!(mat3x3<bool>),
            "Mat3x4I32" => parse_quote!(mat3x4<i32>),
            "Mat3x4U32" => parse_quote!(mat3x4<u32>),
            "Mat3x4F32" => parse_quote!(mat3x4<f32>),
            "Mat3x4F16" => parse_quote!(mat3x4<f16>),
            "Mat3x4Bool" => parse_quote!(mat3x4<bool>),
            "Mat4x2I32" => parse_quote!(mat4x2<i32>),
            "Mat4x2U32" => parse_quote!(mat4x2<u32>),
            "Mat4x2F32" => parse_quote!(mat4x2<f32>),
            "Mat4x2F16" => parse_quote!(mat4x2<f16>),
            "Mat4x2Bool" => parse_quote!(mat4x2<bool>),
            "Mat4x3I32" => parse_quote!(mat4x3<i32>),
            "Mat4x3U32" => parse_quote!(mat4x3<u32>),
            "Mat4x3F32" => parse_quote!(mat4x3<f32>),
            "Mat4x3F16" => parse_quote!(mat4x3<f16>),
            "Mat4x3Bool" => parse_quote!(mat4x3<bool>),
            "Mat4x4I32" => parse_quote!(mat4x4<i32>),
            "Mat4x4U32" => parse_quote!(mat4x4<u32>),
            "Mat4x4F32" => parse_quote!(mat4x4<f32>),
            "Mat4x4F16" => parse_quote!(mat4x4<f16>),
            "Mat4x4Bool" => parse_quote!(mat4x4<bool>),

            _ => {
                let message = format!("Unsupported type in type_to_wgsl: {}", ident.to_string());
                abort!(ident.span(), message)
            }
        }
    }
}
