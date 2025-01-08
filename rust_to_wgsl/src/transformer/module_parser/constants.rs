use quote::ToTokens;
use shared::wgsl_components::{WgslConstAssignment, WgslShaderModuleComponent};
use syn::{ItemConst, visit::Visit};

use crate::{state::ModuleTransformState, transformer::to_wgsl_syntax::convert_to_wgsl};

pub fn find_constants(state: &mut ModuleTransformState) {
    let rust_module = state.rust_module.clone();
    let mut extractor = ConstantsExtractor::new(state);
    extractor.visit_item_mod(&rust_module);
    state.rust_module = rust_module;
}

struct ConstantsExtractor<'a> {
    state: &'a mut ModuleTransformState,
}

impl<'ast> Visit<'ast> for ConstantsExtractor<'ast> {
    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        syn::visit::visit_item_const(self, c);
        self.state
            .result
            .static_consts
            .push(parse_const_assignment(c, self.state));
    }
}

impl<'ast> ConstantsExtractor<'ast> {
    pub fn new(state: &'ast mut ModuleTransformState) -> Self {
        ConstantsExtractor { state }
    }
}

fn parse_const_assignment(
    constant: &ItemConst,
    state: &ModuleTransformState,
) -> WgslConstAssignment {
    WgslConstAssignment {
        code: WgslShaderModuleComponent {
            rust_code: constant.to_token_stream().to_string(),
            wgsl_code: convert_to_wgsl(constant.to_token_stream(), &state).to_string(),
        },
    }
}
