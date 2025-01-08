use crate::{
    state::ModuleTransformState,
    transformer::{allowed_types::AllowedRustTypes, to_wgsl_syntax::convert_to_wgsl},
};
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use shared::wgsl_components::{
    WgslConstAssignment, WgslFunction, WgslShaderModuleComponent, WgslShaderModuleUserPortion,
    WgslType,
};
use syn::{
    Ident, Item, ItemConst, ItemFn, ItemMod,
    spanned::Spanned,
    visit::{self, Visit},
};

pub fn find_main_function(mut state: &mut ModuleTransformState) {
    let module = state.rust_module.clone();
    let mut extractor = MainFunctionsExtractor::new(&mut state);
    extractor.visit_item_mod(&module);
    if state.result.main_function.is_none() {
        abort!(state.rust_module.ident.span(), "No main function found");
    }
    validate_main_function(quote!(&state.result.main_function.unwrap().code.rust_code));
    state.rust_module = module;
}

struct MainFunctionsExtractor<'a> {
    count: usize,
    state: &'a mut ModuleTransformState,
}

impl<'ast> Visit<'ast> for MainFunctionsExtractor<'ast> {
    fn visit_item_fn(&mut self, c: &'ast syn::ItemFn) {
        syn::visit::visit_item_fn(self, c);
        self.count += 1;
        if self.count > 1 {
            abort!(c.sig.ident.span(), "Only one main function is allowed");
        }
        self.state.result.main_function = Some(parse_fn(c, self.state));
    }
}

impl<'ast> MainFunctionsExtractor<'ast> {
    pub fn new(state: &'ast mut ModuleTransformState) -> Self {
        MainFunctionsExtractor { count: 0, state }
    }
}

fn parse_fn(func: &ItemFn, state: &ModuleTransformState) -> WgslFunction {
    WgslFunction {
        code: WgslShaderModuleComponent {
            rust_code: func.to_token_stream().to_string(),
            wgsl_code: convert_to_wgsl(func.to_token_stream(), state).to_string(),
        },
        name: func.sig.ident.to_string(),
    }
}

fn validate_main_function(function: TokenStream) {
    let function = syn::parse2::<ItemFn>(function).unwrap();
    // Check that main has exactly one parameter
    if function.sig.inputs.len() != 1 {
        abort!(
            function.sig.span(),
            "Main function must have exactly one parameter of type WgslGlobalId"
        );
    }
    // Validate the parameter type is WgslGlobalId
    if let syn::FnArg::Typed(pat_type) = &function.sig.inputs[0] {
        if let syn::Type::Path(type_path) = &*pat_type.ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident != "WgslGlobalId" {
                    abort!(
                        pat_type.ty.span(),
                        "Main function parameter must be of type WgslGlobalId"
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
