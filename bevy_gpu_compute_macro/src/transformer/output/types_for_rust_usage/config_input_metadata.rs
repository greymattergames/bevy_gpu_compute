use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    state::ModuleTransformState,
    transformer::custom_types::{
        custom_type::CustomTypeKind, custom_type_idents::CustomTypeIdents,
    },
};

pub fn create_config_input_metadata(state: &ModuleTransformState) -> TokenStream {
    let methods = get_methods(&state);
    quote! {
        pub struct ConfigInputMetadata {
            bytes_per_wgsl_config_type_name: HashMap<String, Vec<u8>>,
        }
        impl ConfigInputDataBuilder {
            pub fn new()-> Self {
                Self {
                    bytes_per_wgsl_config_type_name: HashMap::new(),

                }
            }
            #methods

            pub fn finish(&mut self)-> TypeErasedConfigInputData {
                self.into()
            }
        }
        impl Into<TypeErasedConfigInputData> for ConfigInputDataBuilder {
            fn into(self) -> TypeErasedConfigInputData {
                TypeErasedConfigInputData::new(self.bytes_per_wgsl_config_type_name)
            }
        }
        impl Into<TypeErasedConfigInputData> for &mut ConfigInputDataBuilder {
            fn into(self) -> TypeErasedConfigInputData {
                TypeErasedConfigInputData::new(self.bytes_per_wgsl_config_type_name.clone())
            }
        }
    }
}
fn get_methods(state: &ModuleTransformState) -> TokenStream {
    state
        .custom_types
        .as_ref()
        .unwrap()
        .iter()
        .filter(|c| c.kind == CustomTypeKind::Uniform)
        .map(|c| single_method(c.name.clone()))
        .collect()
}
fn single_method(custom_type_name: CustomTypeIdents) -> TokenStream {
    let method_name: Ident = format_ident!("set_{}", custom_type_name.snake_case);
    let type_pascal_case: Ident = custom_type_name.name.clone();
    let string_key: String = format!("{}", custom_type_name.name.to_string());
    quote! {
        pub fn #method_name(&mut self, data: #type_pascal_case) -> &mut Self {
            self.bytes_per_wgsl_config_type_name
            .insert(#string_key .to_string(), bytemuck::bytes_of(&data).to_vec());
        self
        }
    }
}
