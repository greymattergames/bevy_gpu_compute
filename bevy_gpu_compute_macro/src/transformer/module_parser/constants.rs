use bevy_gpu_compute_core::wgsl::shader_sections::{
    WgslConstAssignment, WgslShaderModuleSectionCode,
};
use quote::ToTokens;
use syn::{ItemConst, visit::Visit};

use crate::{state::ModuleTransformState, transformer::to_wgsl_syntax::convert_file_to_wgsl};

pub fn find_constants(state: &mut ModuleTransformState) {
    let rust_module = state.rust_module.clone();
    let mut extractor = ConstantsExtractor::new(state, false);
    extractor.visit_item_mod(&rust_module);
    let module_for_cpu = state.rust_module_for_cpu.clone();
    let mut extractor_for_cpu = ConstantsExtractor::new(state, true);
    extractor_for_cpu.visit_item_mod(&module_for_cpu);
    state.rust_module = rust_module;
}

struct ConstantsExtractor<'a> {
    state: &'a mut ModuleTransformState,
    for_cpu: bool,
}

impl<'ast> Visit<'ast> for ConstantsExtractor<'ast> {
    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        syn::visit::visit_item_const(self, c);
        if self.for_cpu {
            self.state
                .result_for_cpu
                .static_consts
                .push(parse_const_assignment(c, self.state, self.for_cpu));
        } else {
            self.state.result.static_consts.push(parse_const_assignment(
                c,
                self.state,
                self.for_cpu,
            ));
        }
    }
}

impl<'ast> ConstantsExtractor<'ast> {
    pub fn new(state: &'ast mut ModuleTransformState, for_cpu: bool) -> Self {
        ConstantsExtractor { state, for_cpu }
    }
}

fn parse_const_assignment(
    constant: &ItemConst,
    state: &ModuleTransformState,
    for_cpu: bool,
) -> WgslConstAssignment {
    if !for_cpu {
        WgslConstAssignment {
            code: WgslShaderModuleSectionCode {
                rust_code: constant.to_token_stream().to_string(),
                wgsl_code: convert_file_to_wgsl(
                    constant.to_token_stream(),
                    state,
                    "const".to_string(),
                ),
            },
        }
    } else {
        WgslConstAssignment {
            code: WgslShaderModuleSectionCode {
                rust_code: constant.to_token_stream().to_string(),
                wgsl_code: "".to_string(),
            },
        }
    }
}
