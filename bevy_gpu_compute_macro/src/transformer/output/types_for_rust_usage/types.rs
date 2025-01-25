use std::collections::HashMap;

use bevy_gpu_compute_core::wgsl::{
    shader_custom_type_name::ShaderCustomTypeName,
    shader_module::user_defined_portion::WgslShaderModuleUserPortion,
};
use proc_macro_error::abort;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Ident, parse2, visit_mut::VisitMut};

use crate::{
    state::ModuleTransformState,
    transformer::{
        custom_types::custom_type::CustomTypeKind,
        output::types_for_rust_usage::{
            config_input_data_builder::create_config_input_data_builder,
            input_data_builder::create_input_data_builder,
            make_types_public::MakeTypesPublicTransformer,
            max_output_lengths_builder::create_max_output_lengths_builder,
            output_data_builder::create_output_data_builder,
        },
    },
};

use super::{
    make_types_pod::MakeTypesPodTransformer, remove_internal_attributes::remove_internal_attributes,
};

pub fn define_types_for_use_in_rust_and_set_binding_numbers(
    state: &mut ModuleTransformState,
) -> TokenStream {
    let user_types = user_defined_types(state);
    // order needs to be consistent -> input_configs -> input_arrays -> output_arrays
    let mut binding_num_counter: u32 = 0;
    let mut binding_numbers_by_variable_name: HashMap<String, u32> = HashMap::new();
    let uniforms: TokenStream = uniform_types(
        &mut binding_num_counter,
        &mut binding_numbers_by_variable_name,
        state,
    );
    let input_arrays = input_array_types(
        &mut binding_num_counter,
        &mut binding_numbers_by_variable_name,
        state,
    );
    let output_arrays = output_array_types(
        &mut binding_num_counter,
        &mut binding_numbers_by_variable_name,
        state,
    );
    let max_output_lengths_builder = create_max_output_lengths_builder(state);
    let config_input_data_builder = create_config_input_data_builder(state);
    let input_data_builder = create_input_data_builder(state);
    let output_data_builder = create_output_data_builder(state);
    state.result.binding_numbers_by_variable_name = Some(binding_numbers_by_variable_name);
    quote!(
        /// user types
    #user_types
        /// uniforms
    #uniforms
        /// input arrays
    #input_arrays
        /// output types
    #output_arrays
        /// public facing types for use by library



        /// For passing as a generic argument in the user-facing api, the user should not need to know anything about what "Types" contains
        pub struct Types;
        impl TypesSpec for Types {
            type ConfigInputTypes = _ConfigInputTypes;
            type InputArrayTypes = _InputArrayTypes;
            type OutputArrayTypes = _OutputArrayTypes;
        }

        #max_output_lengths_builder

        #config_input_data_builder

        #input_data_builder

        #output_data_builder


    )
}

pub fn user_defined_types(state: &ModuleTransformState) -> TokenStream {
    let mut publicifier = MakeTypesPublicTransformer {};
    let mut podifier = MakeTypesPodTransformer {};
    let custom_types = remove_internal_attributes(
        state
            .custom_types
            .as_ref()
            .unwrap()
            .iter()
            .map(|c| {
                // get item
                if c.kind == CustomTypeKind::ArrayLengthVariable {
                    return "".to_string();
                }
                let s = c.rust_code.clone();
                let mut item = parse2::<syn::Item>(s);
                if let Err(e) = item {
                    let message = format!(
                        "Error parsing custom type: {:?}, with custom type: {:?}",
                        e, c
                    );
                    abort!(Span::call_site(), message);
                }
                // make public
                publicifier.visit_item_mut(item.as_mut().unwrap());
                podifier.visit_item_mut(item.as_mut().unwrap());
                // stringify
                let string: String = item.unwrap().to_token_stream().to_string();
                string
            })
            .collect::<Vec<String>>()
            .join("\n"),
    );
    custom_types.parse().unwrap()
}

pub fn uniform_types(
    binding_num_counter: &mut u32,
    binding_numbers_by_variable_name: &mut HashMap<String, u32>,

    state: &ModuleTransformState,
) -> TokenStream {
    let obj: WgslShaderModuleUserPortion = state.result.clone();
    let uniforms = obj.uniforms;
    let uniforms_token_streams: TokenStream = uniforms
        .iter()
        .map(|uniform| {
            *binding_num_counter += 1;
            binding_numbers_by_variable_name.insert(uniform.name.uniform(), *binding_num_counter);
            get_single_input_type_metadata(*binding_num_counter, uniform.name.name())
        })
        .collect();

    quote!(

    pub struct _ConfigInputTypes {}
    impl InputTypesMetadataTrait for _ConfigInputTypes {
        fn get_all()-> Vec<InputTypeMetadata> {
            vec![
                #uniforms_token_streams
            ]
        }
    })
}

fn get_single_input_type_metadata(binding_num: u32, input_type: &str) -> TokenStream {
    let ident = Ident::new(input_type, Span::call_site());
    quote!(
        InputTypeMetadata {
            bytes: std::mem::size_of::<#ident>(),
            binding_number: #binding_num,
            name: ShaderCustomTypeName::new(#input_type ),
        },
    )
}
fn get_single_output_type_metadata(
    binding_num: &mut u32,
    binding_numbers_by_variable_name: &mut HashMap<String, u32>,
    type_name: &ShaderCustomTypeName,
    include_count: bool,
) -> TokenStream {
    let ident = Ident::new(type_name.name(), Span::call_site());
    let next_binding_num = *binding_num + 1;
    binding_numbers_by_variable_name.insert(type_name.output_array().to_string(), *binding_num);
    let string_type_name = type_name.name();
    let res = quote!(
        OutputTypeMetadata {
            bytes: std::mem::size_of::<#ident>(),
            binding_number: #binding_num,
            include_count: #include_count,
            count_binding_number: Some(#next_binding_num),
            name: ShaderCustomTypeName::new(#string_type_name ),
        },
    );
    if include_count {
        *binding_num += 1;
        binding_numbers_by_variable_name.insert(type_name.counter().to_string(), *binding_num);
    }
    res
}

pub fn input_array_types(
    binding_num_counter: &mut u32,
    binding_numbers_by_variable_name: &mut HashMap<String, u32>,

    state: &ModuleTransformState,
) -> TokenStream {
    let obj: WgslShaderModuleUserPortion = state.result.clone();
    let input_arrays = obj.input_arrays;
    let input_array_token_streams: TokenStream = input_arrays
        .iter()
        .map(|in_arr| {
            *binding_num_counter += 1;
            binding_numbers_by_variable_name
                .insert(in_arr.item_type.name.input_array(), *binding_num_counter);
            get_single_input_type_metadata(*binding_num_counter, in_arr.item_type.name.name())
        })
        .collect();

    quote!(

        pub struct _InputArrayTypes {}
        impl InputTypesMetadataTrait for _InputArrayTypes {
            fn get_all() -> Vec<InputTypeMetadata> {
                vec![
                    #input_array_token_streams
                ]
            }
        }
    )
}

pub fn output_array_types(
    binding_num_counter: &mut u32,
    binding_numbers_by_variable_name: &mut HashMap<String, u32>,

    state: &ModuleTransformState,
) -> TokenStream {
    let obj: WgslShaderModuleUserPortion = state.result.clone();
    let output_arrays = obj.output_arrays;
    let output_array_token_streams: TokenStream = output_arrays
        .iter()
        .map(|out_arr| {
            *binding_num_counter += 1;
            get_single_output_type_metadata(
                binding_num_counter,
                binding_numbers_by_variable_name,
                &out_arr.item_type.name,
                out_arr.atomic_counter_name.is_some(),
            )
        })
        .collect();

    quote!(

        pub struct _OutputArrayTypes {}
        impl OutputTypesMetadataTrait for _OutputArrayTypes {
            fn get_all() -> Vec<OutputTypeMetadata> {
                vec![
                    #output_array_token_streams
                ]
            }
        }
    )
}
