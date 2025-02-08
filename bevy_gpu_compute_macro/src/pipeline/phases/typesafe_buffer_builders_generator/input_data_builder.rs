use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::pipeline::{
    phases::custom_type_collector::custom_type::CustomType,
    phases::custom_type_collector::{
        custom_type::CustomTypeKind, custom_type_idents::CustomTypeIdents,
    },
};

pub fn create_input_data_builder(custom_types: &Vec<CustomType>) -> TokenStream {
    let methods = get_methods(custom_types);
    quote! {
        pub struct InputDataBuilder {
            bytes_per_wgsl_input_type_name: HashMap<String, Vec<u8>>,
            lengths_per_wgsl_input_type_name: HashMap<String, usize>,
        }
        impl InputDataBuilder {
            pub fn new()-> Self {
                Self {
                    bytes_per_wgsl_input_type_name: HashMap::new(),
                    lengths_per_wgsl_input_type_name: HashMap::new(),

                }
            }
            #methods

            pub fn finish(&mut self)-> TypeErasedArrayInputData {
                self.into()
            }
        }
        impl Into<TypeErasedArrayInputData> for InputDataBuilder {
            fn into(self) -> TypeErasedArrayInputData {
                TypeErasedArrayInputData::new(self.bytes_per_wgsl_input_type_name, self.lengths_per_wgsl_input_type_name)
            }
        }
        impl Into<TypeErasedArrayInputData> for &mut InputDataBuilder {
            fn into(self) -> TypeErasedArrayInputData {
                TypeErasedArrayInputData::new(self.bytes_per_wgsl_input_type_name.clone(), self.lengths_per_wgsl_input_type_name.clone())
            }
        }
    }
}
fn get_methods(custom_types: &Vec<CustomType>) -> TokenStream {
    custom_types
        .iter()
        .filter(|c| c.kind == CustomTypeKind::InputArray)
        .map(|c| single_method(c.name.clone()))
        .collect()
}
fn single_method(custom_type_name: CustomTypeIdents) -> TokenStream {
    let method_name: Ident = format_ident!("set_{}", custom_type_name.snake_case);
    let type_pascal_case: Ident = custom_type_name.name.clone();
    let string_key: String = format!("{}", custom_type_name.name);
    quote! {
        pub fn #method_name(&mut self, data: Vec<#type_pascal_case>) -> &mut Self {
            let length = data.len();
            self.bytes_per_wgsl_input_type_name
            .insert(#string_key .to_string(), bytemuck::cast_slice(&data).to_vec());
        self.lengths_per_wgsl_input_type_name
            .insert(#string_key .to_string(), length);
        self
        }
    }
}
