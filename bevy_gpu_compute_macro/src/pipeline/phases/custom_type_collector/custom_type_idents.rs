use bevy_gpu_compute_core::wgsl::shader_custom_type_name::ShaderCustomTypeName;
use proc_macro2::Span;
use quote::format_ident;
use syn::Ident;

#[derive(Clone, Debug)]

pub struct CustomTypeIdents {
    pub name: Ident,
    pub upper: Ident,
    pub lower: Ident,
    pub snake_case: Ident,
}
impl CustomTypeIdents {
    pub fn new(name: &Ident) -> Self {
        let upper = Ident::new(&name.to_string().to_uppercase(), Span::call_site());
        let lower = Ident::new(&name.to_string().to_lowercase(), Span::call_site());
        let snake_case = Self::pascal_case_to_snake_case(&name.to_string());
        Self {
            name: name.clone(),
            upper,
            lower,
            snake_case,
        }
    }
    pub fn eq(&self, other: &Ident) -> bool {
        self.name == *other
    }
    pub fn uniform(&self) -> &Ident {
        &self.lower
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
    fn pascal_case_to_snake_case(pascal_case: &str) -> Ident {
        let mut snake_case = String::new();
        for (i, c) in pascal_case.chars().enumerate() {
            if c.is_uppercase() && i != 0 {
                snake_case.push('_');
            }
            snake_case.push(c.to_lowercase().next().unwrap());
        }
        format_ident!("{}", snake_case)
    }
}

impl From<CustomTypeIdents> for ShaderCustomTypeName {
    fn from(val: CustomTypeIdents) -> Self {
        ShaderCustomTypeName::new(&val.name.to_string())
    }
}
