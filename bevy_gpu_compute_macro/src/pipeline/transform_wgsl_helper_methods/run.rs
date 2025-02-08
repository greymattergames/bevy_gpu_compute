use proc_macro2::TokenStream;

use syn::{
    Expr, ItemFn, ItemMod, parse_quote,
    visit::Visit,
    visit_mut::{self, VisitMut},
};

use crate::pipeline::{
    custom_types::custom_type::CustomType,
    transform_wgsl_helper_methods::{
        helper_method::WgslHelperMethod, to_expanded_format::ToExpandedFormat,
    },
};

use super::{
    erroneous_usage_finder::ErroneousUsageFinder, parse::parse_possible_wgsl_helper,
    to_expanded_format_for_cpu::ToExpandedFormatForCpu,
};

/// Rust's normal type checking will ensure that these helper functions are using correctly defined types
pub fn transform_wgsl_helper_methods(
    custom_types: &Option<Vec<CustomType>>,
    rust_module: &mut ItemMod,
    for_cpu: bool,
) {
    assert!(custom_types.is_some(), "Allowed types must be defined");
    let custom_types = if let Some(ct) = &custom_types {
        ct
    } else {
        panic!("Allowed types must be set before transforming helper functions");
    };
    let mut converter = WgslHelperExpressionConverter::new(custom_types, for_cpu);
    converter.visit_item_mod_mut(rust_module);
    if !for_cpu {
        let mut error_finder = ErroneousUsageFinder::new(custom_types);
        error_finder.visit_item_mod(rust_module);
    }
}

struct WgslHelperExpressionConverter {
    custom_types: Vec<CustomType>,
    in_main_func: bool,
    nesting_level: u32,
    for_cpu: bool,
}

impl VisitMut for WgslHelperExpressionConverter {
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
        if node.sig.ident == "main" && self.nesting_level == 0 {
            self.in_main_func = true;
            self.nesting_level += 1;
            visit_mut::visit_item_fn_mut(self, node);
            self.nesting_level -= 1;
            self.in_main_func = false;
        } else {
            // For any other function, just increment nesting level and continue
            self.nesting_level += 1;
            visit_mut::visit_item_fn_mut(self, node);
            self.nesting_level -= 1;
        }
    }
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if self.nesting_level > 0 {
            let in_main = self.in_main_func && self.nesting_level == 1;
            if let Expr::Call(call) = expr {
                let helper_method = parse_possible_wgsl_helper(call, &self.custom_types);
                if let Some(method) = helper_method {
                    if self.for_cpu {
                        let replacement = process_wgsl_helper_for_cpu(method);
                        *expr = parse_quote!(#replacement);
                    } else {
                        let replacement = process_wgsl_helper(method, in_main);
                        *expr = parse_quote!(#replacement);
                    }
                }
            }
        }
        // Continue visiting child nodes
        visit_mut::visit_expr_mut(self, expr);
    }
}
impl WgslHelperExpressionConverter {
    pub fn new(custom_types: &[CustomType], for_cpu: bool) -> Self {
        Self {
            custom_types: custom_types.to_vec(),
            in_main_func: false,
            nesting_level: 0,
            for_cpu,
        }
    }
}

fn process_wgsl_helper(helper_method: WgslHelperMethod, in_main_func: bool) -> TokenStream {
    if !helper_method
        .method_expander_kind
        .as_ref()
        .unwrap()
        .valid_outside_main()
        && !in_main_func
    {
        panic!(
            "WGSL helpers that read from inputs or write to outputs (`bevy_gpu_compute_core::wgsl_helpers`) can only be used inside the main function. It is technically possible to pass in entire input arrays, configs, or output arrays to helper functions, but considering the performance implications, it is not recommended. Instead interact with your inputs and outputs in the main function and pass in only the necessary data to the helper functions."
        );
    }
    ToExpandedFormat::run(&helper_method)
}
fn process_wgsl_helper_for_cpu(helper_method: WgslHelperMethod) -> TokenStream {
    ToExpandedFormatForCpu::run(&helper_method)
}
