use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::pipeline::custom_types::{
    custom_type::CustomTypeKind, custom_type_idents::CustomTypeIdents,
};

pub fn create_output_data_builder(custom_types: Vec<CustomType>) -> TokenStream {
    let (fields, init_fields, converters) = get_fields_init_fields_and_converters(custom_types);
    quote! {
        pub struct OutputDataBuilder {
            #fields
        }
        impl From<TypeErasedArrayOutputData> for OutputDataBuilder {
            fn from(out_data: TypeErasedArrayOutputData) ->  Self {
                let mut builder = Self {
                    #init_fields
                };
                #converters
                builder
            }
        }
        impl OutputDataBuilderTrait for OutputDataBuilder {
            fn from(out_data: &TypeErasedArrayOutputData) ->  Self {
                let mut builder = Self {
                    #init_fields
                };
                #converters
                builder
            }
        }
    }
}
fn get_fields_init_fields_and_converters(
    custom_types: Vec<CustomType>,
) -> (TokenStream, TokenStream, TokenStream) {
    custom_types
        .iter()
        .filter(|c| c.kind == CustomTypeKind::OutputArray || c.kind == CustomTypeKind::OutputVec)
        .map(|c| single_field_init_field_and_converter(c.name.clone()))
        .collect()
}
fn single_field_init_field_and_converter(
    custom_type_name: CustomTypeIdents,
) -> (TokenStream, TokenStream, TokenStream) {
    let snake_name: Ident = custom_type_name.snake_case;
    let type_pascal_case: Ident = custom_type_name.name.clone();
    let string_key: String = format!("{}", custom_type_name.name);
    let field = quote! {
        pub #snake_name: Option<Vec<#type_pascal_case>>,
    };
    let init_field = quote! {
        #snake_name: None,
    };
    let message = format!(
        "Byte length not aligned with output type size, for {}",
        string_key
    );
    let converter = quote! {
        let bytes = out_data.get_bytes(#string_key );
        if let Some(b) = bytes{
            if b.len() % std::mem::size_of::<#type_pascal_case>() != 0 {
                panic!( #message );
            }
            if b.len() == 0 {
                builder.#snake_name = Some(Vec::new());
            } else {
                builder.#snake_name = Some(bytemuck::cast_slice(b).to_vec());
            }
        }
    };
    (field, init_field, converter)
}
