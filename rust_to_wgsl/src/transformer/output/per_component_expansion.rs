use proc_macro2::TokenStream;
use quote::quote;
use shared::{
    custom_type_name::CustomTypeName,
    wgsl_components::{
        WgslConstAssignment, WgslFunction, WgslInputArray, WgslOutputArray,
        WgslShaderModuleComponent, WgslType,
    },
};

pub struct ToStructInitializer {}

impl ToStructInitializer {
    pub fn wgsl_shader_module_component(c: WgslShaderModuleComponent) -> TokenStream {
        let r = c.rust_code;
        let w = c.wgsl_code;
        quote!(
            WgslShaderModuleComponent {
                rust_code: (#r).to_string(),
                wgsl_code: (#w).to_string(),
            }
        )
    }

    pub fn wgsl_type(c: WgslType) -> TokenStream {
        let n = ToStructInitializer::custom_type_name(c.name);
        let c = ToStructInitializer::wgsl_shader_module_component(c.code);
        quote!(
            WgslType {
                name: #n,
                code: #c,
            }
        )
        .into()
    }

    pub fn custom_type_name(c: CustomTypeName) -> TokenStream {
        let n = c.name();
        quote!(
            CustomTypeName::new(#n)
        )
    }

    pub fn wgsl_function(c: WgslFunction) -> TokenStream {
        let n = c.name;
        let c = ToStructInitializer::wgsl_shader_module_component(c.code);
        quote!(
            WgslFunction {
                name: (#n).to_string(),
                code: #c
            }
        )
    }

    pub fn wgsl_const_assignment(c: WgslConstAssignment) -> TokenStream {
        let c = ToStructInitializer::wgsl_shader_module_component(c.code);
        quote!(
            WgslConstAssignment {
                code: #c,
            }
        )
    }

    pub fn wgsl_input_array(c: WgslInputArray) -> TokenStream {
        let i = ToStructInitializer::wgsl_type(c.item_type);
        quote!(
            WgslInputArray {
                item_type: #i,
            }
        )
    }

    pub fn wgsl_output_array(c: WgslOutputArray) -> TokenStream {
        let i = ToStructInitializer::wgsl_type(c.item_type);
        let ac: TokenStream = c
            .atomic_counter_name
            .as_ref()
            .map_or("None".to_string(), |counter| {
                format!("Some(\"{}\".to_string())", counter)
            })
            .to_string()
            .parse()
            .unwrap();
        quote!(
            WgslOutputArray {
                item_type: #i,
                atomic_counter_name: #ac

            }
        )
    }
}
