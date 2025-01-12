use std::alloc::Global;

use proc_macro2::TokenStream;
use quote::format_ident;
use shared::wgsl_components::{WgslShaderModuleComponent, WgslType};
use syn::{Attribute, Ident};

use crate::{state::ModuleTransformState, transformer::to_wgsl_syntax::convert_to_wgsl};

use super::custom_type_idents::CustomTypeIdents;

#[derive(PartialEq, Clone, Debug)]
pub enum CustomTypeKind {
    GpuOnlyHelperType,
    Uniform,
    InputArray,
    OutputArray,
    OutputVec,
}

impl From<&Vec<Attribute, Global>> for CustomTypeKind {
    fn from(attrs: &Vec<Attribute, Global>) -> Self {
        for attr in attrs {
            if attr.path().is_ident("wgsl_config") {
                return CustomTypeKind::Uniform;
            } else if attr.path().is_ident("wgsl_input_array") {
                return CustomTypeKind::InputArray;
            } else if attr.path().is_ident("wgsl_output_array") {
                return CustomTypeKind::OutputArray;
            } else if attr.path().is_ident("wgsl_output_vec") {
                return CustomTypeKind::OutputVec;
            }
        }
        CustomTypeKind::GpuOnlyHelperType
    }
}
#[derive(Clone, Debug)]
pub struct CustomType {
    pub name: CustomTypeIdents,
    pub kind: CustomTypeKind,
    pub rust_code: TokenStream,
}
impl CustomType {
    pub fn new(name: &Ident, kind: CustomTypeKind, type_def_code: TokenStream) -> Self {
        Self {
            name: CustomTypeIdents::new(name),
            kind,
            rust_code: type_def_code,
        }
    }
    pub fn into_wgsl_type(self, state: &ModuleTransformState) -> WgslType {
        WgslType {
            name: self.name.into(),
            code: WgslShaderModuleComponent {
                rust_code: self.rust_code.to_string(),
                wgsl_code: convert_to_wgsl(self.rust_code, &state),
            },
        }
    }
}
