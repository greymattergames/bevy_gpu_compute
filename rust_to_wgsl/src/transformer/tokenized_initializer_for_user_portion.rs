use proc_macro2::TokenStream;

use proc_macro_error::abort;
use shared::wgsl_components::{SelfToStructInitializer, WgslShaderModuleUserPortion};

use crate::state::ModuleTransformState;
pub fn convert_wgsl_shader_module_user_portion_into_tokenized_initializer_code(
    state: &ModuleTransformState,
) -> TokenStream {
    let obj: WgslShaderModuleUserPortion = state.result.clone();
    let static_consts = obj
        .static_consts
        .into_iter()
        .map(|const_assignment| const_assignment.to_struct_initializer())
        .collect::<Vec<_>>()
        .join(",");

    let helper_types = obj
        .helper_types
        .into_iter()
        .map(|type_def| type_def.to_struct_initializer())
        .collect::<Vec<_>>()
        .join(",");

    let uniforms = obj
        .uniforms
        .into_iter()
        .map(|uniform| uniform.to_struct_initializer())
        .collect::<Vec<_>>()
        .join(",");

    let input_arrays = obj
        .input_arrays
        .into_iter()
        .map(|array| array.to_struct_initializer())
        .collect::<Vec<_>>()
        .join(",");

    let output_arrays = obj
        .output_arrays
        .into_iter()
        .map(|output_array| output_array.to_struct_initializer())
        .collect::<Vec<_>>()
        .join(",");

    let helper_functions = obj
        .helper_functions
        .into_iter()
        .map(|func| func.to_struct_initializer())
        .collect::<Vec<_>>()
        .join(",");

    let main_function = obj
        .main_function
        .map_or("None".to_string(), |func| func.to_struct_initializer());

    let module_ident = if let Some(c) = &state.module_ident {
        c
    } else {
        abort!(
            state.rust_module.ident.span(),
            "No module ident found in transform state"
        );
    };
    let module_visibility = if let Some(c) = &state.module_visibility {
        c
    } else {
        abort!(
            state.rust_module.ident.span(),
            "No module visibility found in transform state"
        );
    };
    let initialization_code = format!(
        "{} mod {} {{
        use shared::wgsl_components::*; //todo, make this less brittle
        use shared::custom_type_name::*;

        pub fn parsed () -> WgslShaderModuleUserPortion 
        {{
        WgslShaderModuleUserPortion {{
            static_consts: [{}].into(),
            helper_types: [{}].into(),
            uniforms: [{}].into(),
            input_arrays: [{}].into(),
            output_arrays: [{}].into(),
            helper_functions: [{}].into(),
            main_function: Some({}),
        }}
        }}
        }}",
        module_visibility,
        module_ident,
        static_consts,
        helper_types,
        uniforms,
        input_arrays,
        output_arrays,
        helper_functions,
        main_function
    );

    if let Ok(parsed) = initialization_code.parse() {
        return parsed;
    } else {
        abort!(
            state.rust_module.ident.span(),
            "Failed to parse tokenized initializer code"
        );
    }
}
