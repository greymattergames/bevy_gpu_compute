use proc_macro2::TokenStream;

use crate::third_crate::wgsl_components::WgslShaderModuleUserPortion;
pub fn convert_wgsl_shader_module_user_portion_into_tokenized_initializer_code(
    obj: WgslShaderModuleUserPortion,
) -> TokenStream {
    let static_consts = obj
        .static_consts
        .into_iter()
        .map(|const_assignment| {
            format!(
                "WgslConstAssignment {{
                assigner_keyword: \"{}\".to_string(),
                var_name: \"{}\".to_string(),
                var_type: WgslType {{
                    name: \"{}\".to_string(),
                    wgsl: \"{}\".to_string(),
                }},
                value: \"{}\".to_string(),
            }}",
                const_assignment.assigner_keyword,
                const_assignment.var_name,
                const_assignment.var_type.name,
                const_assignment.var_type.wgsl,
                const_assignment.value
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let helper_types = obj
        .helper_types
        .into_iter()
        .map(|type_def| {
            format!(
                "WgslType {{
                name: \"{}\".to_string(),
                wgsl: \"{}\".to_string(),
            }}",
                type_def.name, type_def.wgsl
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let uniforms = obj
        .uniforms
        .into_iter()
        .map(|uniform| {
            format!(
                "WgslType {{
                name: \"{}\".to_string(),
                wgsl: \"{}\".to_string(),
            }}",
                uniform.name, uniform.wgsl
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let input_arrays = obj
        .input_arrays
        .into_iter()
        .map(|array| {
            format!(
                "WgslArray {{
                type_name: \"{}\".to_string(),
                item_type: WgslType {{
                    name: \"{}\".to_string(),
                    wgsl: \"{}\".to_string(),
                }},
                length: {},
            }}",
                array.type_name, array.item_type.name, array.item_type.wgsl, array.length
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let output_arrays = obj
        .output_arrays
        .into_iter()
        .map(|output_array| {
            format!(
                "WgslOutputArray {{
                arr: WgslArray {{
                    type_name: \"{}\".to_string(),
                    item_type: WgslType {{
                        name: \"{}\".to_string(),
                        wgsl: \"{}\".to_string(),
                    }},
                    length: {},
                }},
                atomic_counter: {},
            }}",
                output_array.arr.type_name,
                output_array.arr.item_type.name,
                output_array.arr.item_type.wgsl,
                output_array.arr.length,
                output_array.atomic_counter
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let helper_functions = obj
        .helper_functions
        .into_iter()
        .map(|func| {
            format!(
                "WgslFunction {{
                name: \"{}\".to_string(),
                wgsl_definition: \"{}\".to_string(),
            }}",
                func.name, func.wgsl_definition
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let main_function = obj.main_function.map_or("None".to_string(), |func| {
        format!(
            "Some(WgslFunction {{
                name: \"{}\".to_string(),
                wgsl_definition: \"{}\".to_string(),
            }})",
            func.name, func.wgsl_definition
        )
    });

    let initialization_code = format!(
        "{} mod {} {{
        use crate::third_crate::wgsl_components::*;

        pub fn parsed () -> WgslShaderModuleUserPortion 
        {{
        WgslShaderModuleUserPortion {{
            module_visibility: \"{}\".to_string(),
            module_ident: \"{}\".to_string(),
            static_consts: vec![{}],
            helper_types: vec![{}],
            uniforms: vec![{}],
            input_arrays: vec![{}],
            output_arrays: vec![{}],
            helper_functions: vec![{}],
            main_function: {},
        }}
        }}
        }}",
        obj.module_visibility,
        obj.module_ident,
        obj.module_visibility,
        obj.module_ident,
        static_consts,
        helper_types,
        uniforms,
        input_arrays,
        output_arrays,
        helper_functions,
        main_function
    );

    initialization_code.parse().unwrap()
}
