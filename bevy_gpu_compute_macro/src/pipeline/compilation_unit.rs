use bevy_gpu_compute_core::wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion;
use proc_macro2::TokenStream;

use super::{
    compilation_metadata::CompilationMetadata,
    phases::custom_type_collector::custom_type::CustomType,
};

pub struct CompilationUnit {
    original_rust_module: syn::ItemMod,
    rust_module_for_cpu: Option<syn::ItemMod>,
    rust_module_for_gpu: Option<syn::ItemMod>,
    compiled_tokens: Option<TokenStream>,
    metadata: CompilationMetadata,
}

impl CompilationUnit {
    pub fn new(original_rust_module: syn::ItemMod) -> Self {
        CompilationUnit {
            original_rust_module,
            rust_module_for_cpu: None,
            rust_module_for_gpu: None,
            compiled_tokens: None,
            metadata: CompilationMetadata {
                custom_types: None,
                wgsl_module_user_portion: None,
                typesafe_buffer_builders: None,
            },
        }
    }
    pub fn rust_module_for_gpu(&self) -> &syn::ItemMod {
        if self.rust_module_for_gpu.is_none() {
            panic!("rust_module_for_gpu is not set");
        }
        self.rust_module_for_gpu.as_ref().unwrap()
    }
    pub fn rust_module_for_cpu(&self) -> &syn::ItemMod {
        if self.rust_module_for_cpu.is_none() {
            panic!("rust_module_for_cpu is not set");
        }
        self.rust_module_for_cpu.as_ref().unwrap()
    }
    pub fn set_custom_types(&mut self, custom_types: Vec<CustomType>) {
        self.metadata.custom_types = Some(custom_types);
    }
    pub fn custom_types(&self) -> &Vec<CustomType> {
        if self.metadata.custom_types.is_none() {
            panic!("custom_types is not set");
        }
        self.metadata.custom_types.as_ref().unwrap()
    }
    pub fn original_rust_module(&self) -> &syn::ItemMod {
        &self.original_rust_module
    }
    pub fn set_rust_module_for_gpu(&mut self, rust_module_for_gpu: syn::ItemMod) {
        self.rust_module_for_gpu = Some(rust_module_for_gpu);
    }
    pub fn compiled_tokens(&self) -> &Option<TokenStream> {
        &self.compiled_tokens
    }
    pub fn set_compiled_tokens(&mut self, compiled_tokens: TokenStream) {
        self.compiled_tokens = Some(compiled_tokens);
    }
    pub fn set_rust_module_for_cpu(&mut self, rust_module_for_cpu: syn::ItemMod) {
        self.rust_module_for_cpu = Some(rust_module_for_cpu);
    }
    pub fn set_wgsl_module_user_portion(
        &mut self,
        wgsl_module_user_portion: WgslShaderModuleUserPortion,
    ) {
        self.metadata.wgsl_module_user_portion = Some(wgsl_module_user_portion);
    }
    pub fn wgsl_module_user_portion(&self) -> &WgslShaderModuleUserPortion {
        if self.metadata.wgsl_module_user_portion.is_none() {
            panic!("wgsl_module_user_portion is not set");
        }
        self.metadata.wgsl_module_user_portion.as_ref().unwrap()
    }

    pub fn set_typesafe_buffer_builders(&mut self, typesafe_buffer_builders: TokenStream) {
        self.metadata.typesafe_buffer_builders = Some(typesafe_buffer_builders);
    }
    pub fn typesafe_buffer_builders(&self) -> &TokenStream {
        if self.metadata.typesafe_buffer_builders.is_none() {
            panic!("typesafe_buffer_builders is not set");
        }
        self.metadata.typesafe_buffer_builders.as_ref().unwrap()
    }
}
