use proc_macro_error::abort;
use syn::{spanned::Spanned, visit::Visit};

pub struct DocCommentRemover {}
impl<'ast> Visit<'ast> for DocCommentRemover {
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
