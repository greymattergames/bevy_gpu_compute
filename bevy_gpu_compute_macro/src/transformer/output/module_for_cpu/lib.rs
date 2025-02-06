use crate::state::ModuleTransformState;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{FnArg, Ident, ItemFn};

pub fn generate_module_for_cpu_usage(state: &ModuleTransformState) -> TokenStream {
    // all helper functions need to be publicly exposed here
    // and the main function needs to be rewritten to use parameter inputs, and then exposed here as well
    let helper_funcs = generate_helper_funcs(state);
    let main_func = generate_main_func(state);
    let consts = generate_module_level_consts(state);
    quote!(
        pub mod on_cpu {
            use super::*;
            use bevy_gpu_compute_core::wgsl_helpers::*;
            #consts
            #helper_funcs
            #main_func
        }
    )
}

fn generate_helper_funcs(state: &ModuleTransformState) -> TokenStream {
    state
        .result_for_cpu
        .helper_functions
        .iter()
        .map(|func| {
            // the original rust code is stored as a string, convert it back to TokenStream
            let code: TokenStream = func.code.rust_code.parse().unwrap();
            code
        })
        .collect()
}
fn generate_module_level_consts(state: &ModuleTransformState) -> TokenStream {
    let mut consts = TokenStream::new();
    state
        .result_for_cpu
        .static_consts
        .iter()
        .for_each(|constant| {
            let rust_code: TokenStream = constant.code.rust_code.parse().unwrap();
            consts.extend(quote!(
                #rust_code
            ));
        });
    consts
}

fn generate_main_func(state: &ModuleTransformState) -> TokenStream {
    // the original rust code is stored as a string, convert it back to TokenStream
    let original_tokens: TokenStream = state
        .result_for_cpu
        .main_function
        .as_ref()
        .unwrap()
        .code
        .rust_code
        .parse()
        .unwrap();
    let mut main_func: ItemFn = syn::parse2(original_tokens).unwrap();
    add_params_to_main(state, &mut main_func);
    add_array_lengths_to_main(state, &mut main_func);
    main_func.to_token_stream()
}

fn add_params_to_main(state: &ModuleTransformState, main_func: &mut ItemFn) {
    // add the uniform and array inputs and outputs to the main function parameters
    state.result_for_cpu.uniforms.iter().for_each(|uniform| {
        // turn into a PatType
        let param_name = Ident::new(uniform.name.uniform().as_str(), Span::call_site());
        let param_type = Ident::new(uniform.name.name().as_str(), Span::call_site());
        // let r: FnArg = syn::parse_quote!(#param_name : #param_type);
        let r: FnArg = syn::parse_quote!( #param_name : #param_type );

        main_func.sig.inputs.push(r);
    });
    state.result_for_cpu.input_arrays.iter().for_each(|array| {
        // turn into a PatType
        let param_name = Ident::new(
            array.item_type.name.input_array().as_str(),
            Span::call_site(),
        );
        let param_type = Ident::new(array.item_type.name.name().as_str(), Span::call_site());
        let r: FnArg = syn::parse_quote!(#param_name : Vec<#param_type>);
        main_func.sig.inputs.push(r);
    });
    state.result_for_cpu.output_arrays.iter().for_each(|array| {
        // turn into a PatType
        let param_name = Ident::new(
            array.item_type.name.output_array().as_str(),
            Span::call_site(),
        );
        let param_type = Ident::new(array.item_type.name.name().as_str(), Span::call_site());
        let r: FnArg = syn::parse_quote!(mut #param_name : &mut Vec<#param_type>);
        main_func.sig.inputs.push(r);
    });
}

fn add_array_lengths_to_main(state: &ModuleTransformState, main_func: &mut ItemFn) {
    // add the array lengths to the main function body before anything else
    state.result_for_cpu.input_arrays.iter().for_each(|array| {
        let var_name = Ident::new(
            array.item_type.name.input_array_length().as_str(),
            Span::call_site(),
        );
        let input_array_name = Ident::new(
            array.item_type.name.input_array().as_str(),
            Span::call_site(),
        );
        main_func.block.stmts.insert(
            0,
            syn::parse_quote!(let #var_name = #input_array_name .len();),
        );
    });
    state.result_for_cpu.output_arrays.iter().for_each(|array| {
        let var_name = Ident::new(
            array.item_type.name.output_array_length().as_str(),
            Span::call_site(),
        );
        let output_array_name = Ident::new(
            array.item_type.name.output_array().as_str(),
            Span::call_site(),
        );
        main_func.block.stmts.insert(
            0,
            syn::parse_quote!(let #var_name = #output_array_name .len();),
        );
    })
}
