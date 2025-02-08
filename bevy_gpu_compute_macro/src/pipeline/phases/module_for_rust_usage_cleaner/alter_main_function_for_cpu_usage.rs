use bevy_gpu_compute_core::wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion;
use proc_macro2::Span;
use syn::{FnArg, Ident, ItemFn, visit_mut::VisitMut};

pub fn mutate_main_function_for_cpu_usage(
    wgsl_shader_module_parsed: &WgslShaderModuleUserPortion,
    rust_module_for_cpu: &mut syn::ItemMod,
) {
    let mut main_func_mutator = MainFunctionMutator {
        wgsl_shader_module_parsed,
    };
    main_func_mutator.visit_item_mod_mut(rust_module_for_cpu);
}

struct MainFunctionMutator<'a> {
    wgsl_shader_module_parsed: &'a WgslShaderModuleUserPortion,
}
impl<'a> VisitMut for MainFunctionMutator<'a> {
    fn visit_item_fn_mut(&mut self, c: &mut syn::ItemFn) {
        syn::visit_mut::visit_item_fn_mut(self, c);
        let name = c.sig.ident.to_string();
        if name != "main" {
            return;
        }
        alter_main_function_for_cpu_usage(&self.wgsl_shader_module_parsed, c);
    }
}

fn alter_main_function_for_cpu_usage(
    wgsl_shader_module_parsed: &WgslShaderModuleUserPortion,
    mut main_func: &mut ItemFn,
) {
    add_params_to_main(wgsl_shader_module_parsed, &mut main_func);
    add_array_lengths_to_main(wgsl_shader_module_parsed, &mut main_func);
}

fn add_params_to_main(
    wgsl_shader_module_parsed: &WgslShaderModuleUserPortion,

    main_func: &mut ItemFn,
) {
    // add the uniform and array inputs and outputs to the main function parameters
    wgsl_shader_module_parsed
        .uniforms
        .iter()
        .for_each(|uniform| {
            // turn into a PatType
            let param_name = Ident::new(uniform.name.uniform().as_str(), Span::call_site());
            let param_type = Ident::new(uniform.name.name().as_str(), Span::call_site());
            // let r: FnArg = syn::parse_quote!(#param_name : #param_type);
            let r: FnArg = syn::parse_quote!( #param_name : #param_type );

            main_func.sig.inputs.push(r);
        });
    wgsl_shader_module_parsed
        .input_arrays
        .iter()
        .for_each(|array| {
            // turn into a PatType
            let param_name = Ident::new(
                array.item_type.name.input_array().as_str(),
                Span::call_site(),
            );
            let param_type = Ident::new(array.item_type.name.name().as_str(), Span::call_site());
            let r: FnArg = syn::parse_quote!(#param_name : Vec<#param_type>);
            main_func.sig.inputs.push(r);
        });
    wgsl_shader_module_parsed
        .output_arrays
        .iter()
        .for_each(|array| {
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

fn add_array_lengths_to_main(
    wgsl_shader_module_parsed: &WgslShaderModuleUserPortion,

    main_func: &mut ItemFn,
) {
    // add the array lengths to the main function body before anything else
    wgsl_shader_module_parsed
        .input_arrays
        .iter()
        .for_each(|array| {
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
    wgsl_shader_module_parsed
        .output_arrays
        .iter()
        .for_each(|array| {
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
