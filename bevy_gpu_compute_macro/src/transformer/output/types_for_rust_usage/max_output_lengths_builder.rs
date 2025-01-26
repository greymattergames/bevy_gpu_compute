use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    state::ModuleTransformState,
    transformer::custom_types::{
        custom_type::CustomTypeKind, custom_type_idents::CustomTypeIdents,
    },
};

pub fn create_max_output_lengths_builder(state: &ModuleTransformState) -> TokenStream {
    let methods = get_methods(state);
    quote! {
        pub struct MaxOutputLengthsBuilder {
            length_per_wgsl_output_type_name: HashMap<String, usize>,
        }
        impl MaxOutputLengthsBuilder{
            pub fn new()-> Self {
                Self {
                    length_per_wgsl_output_type_name: HashMap::new(),
                }
            }
            #methods

            pub fn finish(&mut self)-> MaxOutputLengths {
                self.into()
            }
        }
        impl Into<MaxOutputLengths> for MaxOutputLengthsBuilder {
            fn into(self) -> MaxOutputLengths {
                MaxOutputLengths::new(self.length_per_wgsl_output_type_name)
            }
        }
        impl Into<MaxOutputLengths> for &mut MaxOutputLengthsBuilder {
            fn into(self) -> MaxOutputLengths {
                MaxOutputLengths::new(self.length_per_wgsl_output_type_name.clone())
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
        .filter(|c| c.kind == CustomTypeKind::OutputArray || c.kind == CustomTypeKind::OutputVec)
        .map(|c| single_method(c.name.clone()))
        .collect()
}
fn single_method(custom_type_name: CustomTypeIdents) -> TokenStream {
    let method_name: Ident = format_ident!("set_{}", custom_type_name.snake_case);
    let string_key: String = format!("{}", custom_type_name.name);
    quote! {
        pub fn #method_name(&mut self, length: usize) -> &mut Self {
            self.length_per_wgsl_output_type_name.insert(#string_key .to_string(), length);
            self
        }
    }
}
