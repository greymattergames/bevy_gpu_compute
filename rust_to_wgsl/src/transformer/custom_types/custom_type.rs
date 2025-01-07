use std::alloc::Global;

use quote::format_ident;
use syn::{Attribute, Ident};

#[derive(PartialEq, Clone)]
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
            if attr.path().is_ident("wgsl_uniform") {
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
#[derive(Clone)]
pub struct CustomType {
    pub name: CustomTypeName,
    pub kind: CustomTypeKind,
}
#[derive(Clone)]

pub struct CustomTypeName {
    pub name: Ident,
    pub upper: Ident,
    pub lower: Ident,
}
impl CustomTypeName {
    pub fn new(name: &Ident) -> Self {
        let upper = Ident::new(&name.to_string().to_uppercase(), name.span());
        let lower = Ident::new(&name.to_string().to_lowercase(), name.span());
        Self {
            name: name.clone(),
            upper,
            lower,
        }
    }
    pub fn eq(&self, other: &String) -> bool {
        self.name.to_string() == *other
    }
    pub fn input_array_length(&self) -> Ident {
        format_ident!("{}_INPUT_ARRAY_LENGTH", self.upper)
    }
    pub fn input_array(&self) -> Ident {
        format_ident!("{}_input_array", self.lower)
    }
    pub fn output_array_length(&self) -> Ident {
        format_ident!("{}_OUTPUT_ARRAY_LENGTH", self.upper)
    }
    pub fn output_array(&self) -> Ident {
        format_ident!("{}_output_array", self.lower)
    }
    pub fn counter(&self) -> Ident {
        format_ident!("{}_counter", self.lower)
    }
    pub fn index(&self) -> Ident {
        format_ident!("{}_output_array_index", self.lower)
    }
}
