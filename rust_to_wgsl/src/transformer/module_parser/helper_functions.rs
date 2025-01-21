use quote::ToTokens;
use shared::wgsl_components::{WgslFunction, WgslShaderModuleComponent};
use syn::{ItemFn, visit::Visit};

use crate::{state::ModuleTransformState, transformer::to_wgsl_syntax::convert_file_to_wgsl};

pub fn find_helper_functions(mut state: &mut ModuleTransformState) {
    let module = state.rust_module.clone();
    let mut extractor = HelperFunctionsExtractor::new(&mut state);
    extractor.visit_item_mod(&module);
    state.rust_module = module;
}

struct HelperFunctionsExtractor<'a> {
    state: &'a mut ModuleTransformState,
}

impl<'ast> Visit<'ast> for HelperFunctionsExtractor<'ast> {
    fn visit_item_fn(&mut self, c: &'ast syn::ItemFn) {
        syn::visit::visit_item_fn(self, c);
        if c.sig.ident.to_string() == "main" {
            return;
        }
        // ident from string

        self.state
            .result
            .helper_functions
            .push(parse_fn(c, self.state));
    }
}

impl<'ast> HelperFunctionsExtractor<'ast> {
    pub fn new(state: &'ast mut ModuleTransformState) -> Self {
        HelperFunctionsExtractor { state }
    }
}

fn parse_fn(func: &ItemFn, state: &ModuleTransformState) -> WgslFunction {
    WgslFunction {
        code: WgslShaderModuleComponent {
            rust_code: func.to_token_stream().to_string(),
            wgsl_code: convert_file_to_wgsl(func.to_token_stream(), state, "helper fn".to_string()),
        },
        name: func.sig.ident.to_string(),
    }
}
