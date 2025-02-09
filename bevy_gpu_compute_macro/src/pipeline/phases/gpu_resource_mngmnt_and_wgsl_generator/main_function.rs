use super::to_wgsl_syntax::convert_file_to_wgsl;
use crate::pipeline::phases::custom_type_collector::custom_type::CustomType;
use bevy_gpu_compute_core::wgsl::shader_sections::{WgslFunction, WgslShaderModuleSectionCode};
use proc_macro::Span;
use proc_macro_error::abort;
use quote::ToTokens;
use syn::{ItemFn, ItemMod, spanned::Spanned, visit::Visit};

/// 0: for gpu, 1: for cpu
pub fn parse_main_function(
    rust_module_transformed_for_gpu: &ItemMod,
    custom_types: &Vec<CustomType>,
) -> WgslFunction {
    let mut extractor = MainFunctionsExtractor::new(custom_types);
    extractor.visit_item_mod(rust_module_transformed_for_gpu);

    if let Some(mf) = extractor.result {
        mf
    } else {
        abort!(Span::call_site(), "No main function found");
    }
}

struct MainFunctionsExtractor<'a> {
    count: usize,
    custom_types: &'a Vec<CustomType>,
    result: Option<WgslFunction>,
}

impl<'ast> Visit<'ast> for MainFunctionsExtractor<'ast> {
    fn visit_item_fn(&mut self, c: &'ast syn::ItemFn) {
        syn::visit::visit_item_fn(self, c);
        let name = c.sig.ident.to_string();
        if name != "main" {
            return;
        }
        self.count += 1;
        if self.count > 1 {
            abort!(c.sig.ident.span(), "Only one main function is allowed");
        }

        self.result = Some(parse_main_fn(c, self.custom_types));
    }
}

impl<'ast> MainFunctionsExtractor<'ast> {
    pub fn new(custom_types: &'ast Vec<CustomType>) -> Self {
        MainFunctionsExtractor {
            count: 0,
            custom_types,
            result: None,
        }
    }
}

fn parse_main_fn(func: &ItemFn, custom_types: &Vec<CustomType>) -> WgslFunction {
    validate_main_function(func);
    let func_clone = func.clone();
    // alter the main function argument
    WgslFunction {
        code: WgslShaderModuleSectionCode {
            wgsl_code: alter_global_id_argument(convert_file_to_wgsl(
                func_clone.to_token_stream(),
                custom_types,
                "main".to_string(),
            )),
        },
        name: func_clone.sig.ident.to_string(),
    }
}
/// we have to alter the main function argument to match the wgsl spec by string replace instead of ast manipulation because the new argument is not a valid rust syntax
fn alter_global_id_argument(func_string: String) -> String {
    let match_patterns = [
        "iter_pos: WgslIterationPosition",
        "iter_pos : WgslIterationPosition",
        "iter_pos:WgslIterationPosition",
        "iter_pos:  WgslIterationPosition",
    ];
    let replace_pattern = "@builtin(global_invocation_id) iter_pos: vec3<u32>";
    let mut new_func = func_string.clone();
    let mut found = false;
    for pattern in match_patterns.iter() {
        if new_func.contains(pattern) {
            found = true;
            new_func = new_func.replace(pattern, replace_pattern);
        }
    }
    if !found {
        let error_message = format!(
            "Failed to find main function argument, we are looking for a string that exactly matches 'iter_pos: WgslIterationPosition', found {}",
            new_func
        );
        abort!(Span::call_site(), error_message);
    }
    new_func
}

fn validate_main_function(function: &ItemFn) {
    // Check that main has exactly one parameter
    if function.sig.inputs.len() != 1 {
        abort!(
            function.sig.span(),
            "Main function must have exactly one parameter of type WgslIterationPosition"
        );
    }
    // Validate the parameter type is WgslIterationPosition called "global_id"
    if let syn::FnArg::Typed(pat_type) = &function.sig.inputs[0] {
        match &*pat_type.pat {
            syn::Pat::Ident(_) => {}
            _ => abort!(
                pat_type.pat.span(),
                "Main function parameter must be called 'iter_pos'"
            ),
        }
        if let syn::Type::Path(type_path) = &*pat_type.ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident != "WgslIterationPosition" {
                    abort!(
                        pat_type.ty.span(),
                        "Main function parameter must be of type WgslIterationPosition"
                    );
                }
            }
        }
    }
    // Check return type (should be void/unit)
    if let syn::ReturnType::Type(_, _) = &function.sig.output {
        abort!(
            function.sig.span(),
            "Main function cannot have a return type"
        );
    }
}
