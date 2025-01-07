use proc_macro_error::abort;
use syn::spanned::Spanned;

use crate::transformer::custom_types::custom_type::CustomType;

use super::array::array_to_wgsl;

pub fn type_to_wgsl(ty: &syn::Type, custom_types: &Vec<CustomType>) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let ident = type_path.path.get_ident();
            if let Some(ident) = ident {
                let segment = &type_path.path.segments.first().unwrap();
                let custom_t = custom_types.iter().find(|t| t.name.eq(&ident.to_string()));
                if let Some(_) = custom_t {
                    ident.to_string()
                } else {
                    match ident.to_string().as_str() {
                        "f32" => "f32".to_string(),
                        "i32" => "i32".to_string(),
                        "u32" => "u32".to_string(),
                        "bool" => "bool".to_string(),
                        "Vec2" => handle_vec(segment, "vec2", custom_types),
                        "Vec3" => handle_vec(segment, "vec3", custom_types),
                        "Vec4" => handle_vec(segment, "vec4", custom_types),
                        "Mat2x2" => handle_mat(segment, "mat2x2", custom_types),
                        "Mat3x3" => handle_mat(segment, "mat3x3", custom_types),
                        "Mat4x4" => handle_mat(segment, "mat4x4", custom_types),
                        _ => abort!(ident.span(), "Unsupported type in type_to_wgsl"),
                    }
                }
            } else {
                abort!(type_path.span(), "Type paths are not allowed in wgsl")
            }
        }
        syn::Type::Array(array) => array_to_wgsl(array),
        _ => abort!(ty.span(), "Unsupported type in type_to_wgsl"),
    }
}

fn handle_vec(segment: &syn::PathSegment, name: &str, custom_types: &Vec<CustomType>) -> String {
    {
        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                format!("{}<{}>", name, type_to_wgsl(inner_type, custom_types))
            } else {
                abort!(
                    segment.span(),
                    format!("{} requires a type parameter", name)
                )
            }
        } else {
            abort!(
                segment.span(),
                format!("{} requires a type parameter", name)
            )
        }
    }
}

fn handle_mat(segment: &syn::PathSegment, name: &str, custom_types: &Vec<CustomType>) -> String {
    {
        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                format!("{}<{}>", name, type_to_wgsl(inner_type, custom_types))
            } else {
                abort!(
                    segment.span(),
                    format!("{} requires a type parameter", name)
                )
            }
        } else {
            abort!(
                segment.span(),
                format!("{} requires a type parameter", name)
            )
        }
    }
}
