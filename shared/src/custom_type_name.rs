use proc_macro2::Ident;
use quote::format_ident;

use crate::wgsl_components::SelfToStructInitializer;

#[derive(Clone, Debug)]

pub struct CustomTypeName {
    pub name: Ident,
    pub upper: Ident,
    pub lower: Ident,
}
impl SelfToStructInitializer for CustomTypeName {
    fn to_struct_initializer(&self) -> String {
        format!(
            "CustomTypeName {{
                name: {},
                upper: {},
                lower: {},
            }}",
            self.name.to_string(),
            self.upper.to_string(),
            self.lower.to_string()
        )
    }
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
