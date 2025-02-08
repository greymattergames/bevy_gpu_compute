use bevy_gpu_compute_core::wgsl::shader_sections::{
    WgslConstAssignment, WgslShaderModuleSectionCode,
};
use quote::ToTokens;
use syn::{ItemConst, ItemMod, visit::Visit};

use super::to_wgsl_syntax::convert_file_to_wgsl;
use crate::pipeline::phases::custom_type_collector::custom_type::CustomType;

// todo ensure this only searches the module level, right now its searching within functions as well

pub fn extract_constants(
    rust_module_transformed_for_gpu: &ItemMod,
    custom_types: &Vec<CustomType>,
) -> Vec<WgslConstAssignment> {
    let mut extractor = ConstantsExtractor::new(custom_types);
    extractor.visit_item_mod(&rust_module_transformed_for_gpu);
    extractor.results
}

struct ConstantsExtractor<'a> {
    custom_types: &'a Vec<CustomType>,
    results: Vec<WgslConstAssignment>,
}

impl<'ast> Visit<'ast> for ConstantsExtractor<'ast> {
    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        syn::visit::visit_item_const(self, c);
        self.results
            .push(parse_const_assignment(c, self.custom_types));
    }
}

impl<'ast> ConstantsExtractor<'ast> {
    pub fn new(custom_types: &'ast Vec<CustomType>) -> Self {
        ConstantsExtractor {
            custom_types,
            results: Vec::new(),
        }
    }
}

fn parse_const_assignment(
    constant: &ItemConst,
    custom_types: &Vec<CustomType>,
) -> WgslConstAssignment {
    WgslConstAssignment {
        code: WgslShaderModuleSectionCode {
            wgsl_code: convert_file_to_wgsl(
                constant.to_token_stream(),
                custom_types,
                "const".to_string(),
            ),
        },
    }
}
