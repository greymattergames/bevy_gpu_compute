use proc_macro_error::abort;
use syn::{spanned::Spanned, visit::Visit};

pub fn validate_no_doc_comments(original_rust_module: &syn::ItemMod) {
    let mut checker = DocCommentChecker {};
    checker.visit_item_mod(&original_rust_module);
}
struct DocCommentChecker {}
impl Visit<'_> for DocCommentChecker {
    fn visit_attribute(&mut self, attr: &syn::Attribute) {
        syn::visit::visit_attribute(self, attr);
        if let Some(ident) = attr.path().get_ident() {
            if ident == "doc" {
                abort!(
                    attr.span(),
                    "Doc comments are not allowed in wgsl_to_rust shader modules. Please remove them."
                );
            }
        }
    }
}
