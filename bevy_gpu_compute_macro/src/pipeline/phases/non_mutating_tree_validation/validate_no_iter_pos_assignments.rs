use proc_macro_error::abort;
use syn::{ExprAssign, spanned::Spanned, visit::Visit};

pub fn validate_no_iter_pos_assignments(original_rust_module: &syn::ItemMod) {
    let mut checker = IterPosAssignmentChecker {};
    checker.visit_item_mod(original_rust_module);
}

struct IterPosAssignmentChecker {}
impl<'ast> Visit<'ast> for IterPosAssignmentChecker {
    fn visit_expr_assign(&mut self, c: &'ast syn::ExprAssign) {
        syn::visit::visit_expr_assign(self, c);
        check_for_iter_pos_assignment(c);
    }
}

fn check_for_iter_pos_assignment(assign: &ExprAssign) {
    // Check direct assignments to iter_pos
    if let syn::Expr::Path(path) = &*assign.left {
        if let Some(ident) = path.path.segments.last() {
            if ident.ident == "iter_pos" {
                abort!(assign.span(), "Cannot assign to iter_pos");
            }
        }
    }
    // Check field assignments like iter_pos.x
    if let syn::Expr::Field(field) = &*assign.left {
        if let syn::Expr::Path(path) = &*field.base {
            if let Some(ident) = path.path.segments.last() {
                if ident.ident == "iter_pos" {
                    abort!(assign.span(), "Cannot assign to iter_pos components");
                }
            }
        }
    }
}
