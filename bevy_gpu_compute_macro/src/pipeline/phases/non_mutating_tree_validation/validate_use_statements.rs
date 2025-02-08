use proc_macro_error::abort;
use quote::ToTokens;
use syn::{Item, ItemMod, ItemUse, spanned::Spanned, visit::Visit};

const VALID_USE_STATEMENT_PATHS: [&str; 3] =
    ["wgsl_helpers", "bevy_gpu_compute", "bevy_gpu_compute_macro"];

pub fn validate_use_statements(original_rust_module: &ItemMod) {
    let mut handler = UseStatementHandler {};
    handler.visit_item_mod(original_rust_module);
}

struct UseStatementHandler {}

impl Visit<'_> for UseStatementHandler {
    fn visit_item(&mut self, i: &Item) {
        syn::visit::visit_item(self, i);
        if let Item::Use(use_stmt) = i {
            validate_use_statement(use_stmt);
        }
    }
}

fn validate_use_statement(use_stmt: &ItemUse) {
    let mut single_handler = SingleUseStatementHandler { found: false };
    single_handler.visit_item_use(use_stmt);
    if !single_handler.found {
        let message = format!(
            "Invalid use statement: {:?}. You are only allowed to import from one of these crates: {}",
            use_stmt.to_token_stream().to_string(),
            VALID_USE_STATEMENT_PATHS.join(", ")
        );
        abort!(use_stmt.span(), message);
    }
}

struct SingleUseStatementHandler {
    found: bool,
}

impl Visit<'_> for SingleUseStatementHandler {
    fn visit_use_path(&mut self, i: &syn::UsePath) {
        syn::visit::visit_use_path(self, i);
        if VALID_USE_STATEMENT_PATHS.contains(&i.ident.to_string().as_str()) {
            self.found = true;
        }
    }
    fn visit_use_name(&mut self, i: &syn::UseName) {
        syn::visit::visit_use_name(self, i);
        if VALID_USE_STATEMENT_PATHS.contains(&i.ident.to_string().as_str()) {
            self.found = true;
        }
    }
}
