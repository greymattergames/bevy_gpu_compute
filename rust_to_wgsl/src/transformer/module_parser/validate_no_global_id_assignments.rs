use proc_macro_error::abort;
use syn::{ExprAssign, spanned::Spanned, visit::Visit};

use crate::state::ModuleTransformState;

pub fn check_module_for_global_id_assignment(state: &mut ModuleTransformState) {
    let mut checker = GlobalIdAssignmentChecker {};
    checker.visit_item_mod(&state.rust_module);
}

struct GlobalIdAssignmentChecker {}
impl<'ast> Visit<'ast> for GlobalIdAssignmentChecker {
    fn visit_expr_assign(&mut self, c: &'ast syn::ExprAssign) {
        syn::visit::visit_expr_assign(self, c);
        check_for_global_id_assignment(c);
    }
}

fn check_for_global_id_assignment(assign: &ExprAssign) {
    // Check direct assignments to global_id
    if let syn::Expr::Path(path) = &*assign.left {
        if let Some(ident) = path.path.segments.last() {
            if ident.ident == "global_id" {
                abort!(assign.span(), "Cannot assign to global_id");
            }
        }
    }
    // Check field assignments like global_id.x
    if let syn::Expr::Field(field) = &*assign.left {
        if let syn::Expr::Path(path) = &*field.base {
            if let Some(ident) = path.path.segments.last() {
                if ident.ident == "global_id" {
                    abort!(assign.span(), "Cannot assign to global_id components");
                }
            }
        }
    }
}
