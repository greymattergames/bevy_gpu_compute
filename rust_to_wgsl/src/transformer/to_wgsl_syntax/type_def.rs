use std::collections::HashMap;

use proc_macro_error::abort;
use quote::ToTokens;
use syn::{ItemType, parse_quote, visit::Visit, visit_mut::VisitMut};

use crate::transformer::{allowed_types::WGSL_NATIVE_TYPES, custom_types::custom_type::CustomType};

pub struct TypeDefToWgslTransformer {
    pub replacements: HashMap<String, String>,
}

impl<'ast> Visit<'ast> for TypeDefToWgslTransformer {
    fn visit_item_type(&mut self, t: &syn::ItemType) {
        syn::visit::visit_item_type(self, t);
        // Instead of direct replacement, use placeholder system
        let new_type_def = type_def_to_wgsl(t);
        let existing = t.to_token_stream().to_string();
        // extract everything to the left of the first =
        let expr = existing.split('=').collect::<Vec<&str>>()[0]
            .trim()
            .to_string();
        self.replacements.insert(expr, new_type_def);
    }
}

const UNALLOWED_TYPES_FOR_RENAMING: [&str; 12] = [
    "vec2", "vec3", "vec4", "mat2x2", "mat2x3", "mat2x4", "mat3x2", "mat3x3", "mat3x4", "mat4x2",
    "mat4x3", "mat4x4",
];
fn type_def_to_wgsl(type_def: &ItemType) -> String {
    // ensure that the type is not a custom type
    match *type_def.ty.clone() {
        syn::Type::Path(p) => {
            if let Some(f) = p.path.segments.first() {
                let mtch = UNALLOWED_TYPES_FOR_RENAMING
                    .iter()
                    .find(|t| **t == f.ident.to_string());
                if mtch.is_some() {
                    abort!(
                        f.ident.span(),
                        "Renaming/aliasing helper types like Vec3F32, Mat2x2Bool, etc. is not supported. For example don't do `type MyType = Vec3U32;`. Instead put it in a struct field like `struct MyType = { v: Vec3U32 }`"
                    );
                }
            }
        }
        _ => (),
    }
    let s = format!("alias {} ", type_def.ident.to_string(),);
    s
}
