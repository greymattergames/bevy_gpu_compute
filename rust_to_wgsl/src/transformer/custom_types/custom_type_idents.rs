use proc_macro2::Span;
use quote::format_ident;
use shared::custom_type_name::CustomTypeName;
use syn::Ident;

#[derive(Clone, Debug)]

pub struct CustomTypeIdents {
    pub name: Ident,
    pub upper: Ident,
    pub lower: Ident,
}
impl CustomTypeIdents {
    pub fn new(name: &Ident) -> Self {
        let upper = Ident::new(&name.to_string().to_uppercase(), Span::call_site());
        let lower = Ident::new(&name.to_string().to_lowercase(), Span::call_site());
        Self {
            name: name.clone(),
            upper,
            lower,
        }
    }
    pub fn eq(&self, other: &Ident) -> bool {
        self.name.to_string() == *other.to_string()
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
}

impl Into<CustomTypeName> for CustomTypeIdents {
    fn into(self) -> CustomTypeName {
        CustomTypeName::new(&self.name.to_string())
    }
}
