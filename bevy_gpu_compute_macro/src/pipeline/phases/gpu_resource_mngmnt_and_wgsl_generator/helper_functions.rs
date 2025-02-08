use super::to_wgsl_syntax::convert_file_to_wgsl;
use bevy_gpu_compute_core::wgsl::shader_sections::{WgslFunction, WgslShaderModuleSectionCode};
use quote::ToTokens;
use syn::{ItemFn, ItemMod, visit::Visit};

use crate::pipeline::phases::custom_type_collector::custom_type::CustomType;

pub fn extract_helper_functions(
    rust_module_transformed_for_gpu: &ItemMod,
    custom_types: &Vec<CustomType>,
) -> Vec<WgslFunction> {
    let mut extractor = HelperFunctionsExtractor::new(custom_types);
    extractor.visit_item_mod(rust_module_transformed_for_gpu);
    extractor.results
}

struct HelperFunctionsExtractor<'a> {
    custom_types: &'a Vec<CustomType>,
    results: Vec<WgslFunction>,
}

impl<'ast> Visit<'ast> for HelperFunctionsExtractor<'ast> {
    fn visit_item_fn(&mut self, c: &'ast syn::ItemFn) {
        syn::visit::visit_item_fn(self, c);
        if c.sig.ident == "main" {
            return;
        }
        // ident from string

        self.results.push(parse_fn(c, self.custom_types));
    }
}

impl<'ast> HelperFunctionsExtractor<'ast> {
    pub fn new(custom_types: &'ast Vec<CustomType>) -> Self {
        HelperFunctionsExtractor {
            custom_types,
            results: Vec::new(),
        }
    }
}

fn parse_fn(func: &ItemFn, custom_types: &Vec<CustomType>) -> WgslFunction {
    WgslFunction {
        code: WgslShaderModuleSectionCode {
            wgsl_code: convert_file_to_wgsl(
                func.to_token_stream(),
                custom_types,
                "helper fn".to_string(),
            ),
        },
        name: func.sig.ident.to_string(),
    }
}
