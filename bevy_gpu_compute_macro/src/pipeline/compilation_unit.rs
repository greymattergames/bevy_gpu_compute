use proc_macro::TokenStream;

use super::{
    compilation_metadata::CompilationMetadata,
    phases::custom_type_collector::custom_type::CustomType,
};

pub struct CompilationUnit {
    tokens: TokenStream,
    original_rust_module: syn::ItemMod,
    metadata: CompilationMetadata,
}

impl CompilationUnit {
    pub fn new(
        tokens: TokenStream,
        original_rust_module: syn::ItemMod,
        metadata: CompilationMetadata,
    ) -> Self {
        CompilationUnit {
            tokens,
            original_rust_module,
            metadata,
        }
    }
    pub fn add_custom_types(&mut self, custom_types: Vec<CustomType>) {
        self.metadata.custom_types = custom_types;
    }
    pub fn original_rust_module(&self) -> &syn::ItemMod {
        &self.original_rust_module
    }
}
