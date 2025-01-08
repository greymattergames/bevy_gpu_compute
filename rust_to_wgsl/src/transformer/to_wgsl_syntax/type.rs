use proc_macro_error::abort;
use syn::{PathSegment, TypePath, parse_quote, spanned::Spanned, visit_mut::VisitMut};

use crate::transformer::custom_types::custom_type::CustomType;

use super::array::array_to_wgsl;

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
    let custom_t = custom_types.iter().find(|t| t.name.eq(&ident.to_string()));
    if let Some(_) = custom_t {
        segment.clone()
    } else {
        match ident.to_string().as_str() {
            "f32" => segment.clone(),
            "i32" => segment.clone(),
            "u32" => segment.clone(),
            "bool" => segment.clone(),
            "Vec2" => handle_vec(&segment, "vec2", custom_types),
            "Vec3" => handle_vec(&segment, "vec3", custom_types),
            "Vec4" => handle_vec(&segment, "vec4", custom_types),
            "Mat2x2" => handle_mat(&segment, "mat2x2", custom_types),
            "Mat3x3" => handle_mat(&segment, "mat3x3", custom_types),
            "Mat4x4" => handle_mat(&segment, "mat4x4", custom_types),
            _ => abort!(ident.span(), "Unsupported type in type_to_wgsl"),
        }
    }
}

fn handle_vec(
    segment: &syn::PathSegment,
    name: &str,
    custom_types: &Vec<CustomType>,
) -> PathSegment {
    {
        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                parse_quote!(#name<#inner_type>)
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

fn handle_mat(
    segment: &syn::PathSegment,
    name: &str,
    custom_types: &Vec<CustomType>,
) -> PathSegment {
    {
        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                parse_quote!(#name<#inner_type>)
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
