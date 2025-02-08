use std::alloc::Global;

use bevy_gpu_compute_core::wgsl::shader_sections::{WgslShaderModuleSectionCode, WgslType};
use proc_macro2::TokenStream;
use syn::{Attribute, Ident};

use crate::pipeline::phases::gpu_resource_mngmnt_and_wgsl_generator::to_wgsl_syntax::convert_file_to_wgsl;

use super::custom_type_idents::CustomTypeIdents;

#[derive(PartialEq, Clone, Debug)]
pub enum CustomTypeKind {
    GpuOnlyHelperType,
    Uniform,
    InputArray,
    OutputArray,
    OutputVec,
    ArrayLengthVariable,
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
    pub fn into_wgsl_type(self, custom_types: &Vec<CustomType>) -> WgslType {
        WgslType {
            name: self.name.into(),
            code: WgslShaderModuleSectionCode {
                wgsl_code: convert_file_to_wgsl(
                    self.rust_code,
                    custom_types,
                    "custom_type".to_string(),
                ),
            },
        }
    }
}
