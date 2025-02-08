use syn::{
    Expr,
    visit::{self, Visit},
};

use crate::transformer::custom_types::custom_type::CustomType;

use super::parse::parse_possible_wgsl_helper;

pub struct ErroneousUsageFinder {
    custom_types: Vec<CustomType>,
}
impl ErroneousUsageFinder {
    pub fn new(custom_types: &[CustomType]) -> Self {
        Self {
            custom_types: custom_types.to_vec(),
        }
    }
}
impl Visit<'_> for ErroneousUsageFinder {
    /// This error message relies on `WgslVecInput` being in `bevy_gpu_compute_core::wgsl_helpers`
    /**
    ```rust
    // ensure that the crate structure is what we expect, otherwise the error message will be incorrect
       use bevy_gpu_compute_core::wgsl_helpers::WgslVecInput;
    ```
    */
    fn visit_expr(&mut self, expr: &Expr) {
        visit::visit_expr(self, expr);
        if let Expr::Call(call) = expr {
            let helper_method = parse_possible_wgsl_helper(call, &self.custom_types);
            if helper_method.is_some() {
                panic!(
                    "WGSL Helpers (`bevy_gpu_compute_core::wgsl_helpers`) not allowed outside of functions."
                );
            }
        }
    }
}
