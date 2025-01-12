use proc_macro_error::abort;
use quote::quote;
use syn::{Item, ItemMod, ItemUse, UseTree, spanned::Spanned};

use crate::state::ModuleTransformState;

pub fn handle_use_statements<'a>(state: &mut ModuleTransformState) {
    let mut found_valid_use_statement = false;
    let module_content = if let Some(c) = &state.rust_module.content {
        c
    } else {
        abort!(
            state.rust_module.ident.span(),
            "Shader module must have a body"
        );
    };
    for item in module_content.1.iter() {
        if let Item::Use(use_stmt) = item {
            if !is_valid_use_statement(use_stmt, "wgsl_in_rust_helpers") {
                abort!(
                    use_stmt.span(),
                    "You cannot export anything into a wgsl module, except for the single helper crate this library provides."
                );
            }
            if found_valid_use_statement {
                abort!(
                    use_stmt.span(),
                    "Only one helper crate import is allowed in a wgsl module."
                );
            }
            found_valid_use_statement = true;
        }
    }
    state.rust_module.content = Some((
        module_content.0,
        module_content
            .1
            .iter()
            .filter(|item| !matches!(item, Item::Use(_)))
            .cloned()
            .collect(),
    ));
}

fn is_valid_use_statement(use_stmt: &ItemUse, valid_path_segment: &str) -> bool {
    fn check_path_for_helper(path: &UseTree, valid_path_segment: &str) -> bool {
        match path {
            UseTree::Name(name) => name.ident == valid_path_segment,
            UseTree::Path(path) if path.ident == valid_path_segment => true,
            UseTree::Path(path) => check_path_for_helper(&*path.tree, valid_path_segment),
            _ => false,
        }
    }

    let mut current = &use_stmt.tree;
    loop {
        match current {
            syn::UseTree::Path(path) => {
                if check_path_for_helper(&*path.tree, valid_path_segment) {
                    return true;
                }
                current = &path.tree;
            }
            syn::UseTree::Glob(..) => {
                // Only return true if parent path contained wgsl_in_rust_helpers
                return match &use_stmt.tree {
                    syn::UseTree::Path(path) => {
                        check_path_for_helper(&path.tree, valid_path_segment)
                    }
                    _ => false,
                };
            }
            _ => return false,
        }
    }
}
