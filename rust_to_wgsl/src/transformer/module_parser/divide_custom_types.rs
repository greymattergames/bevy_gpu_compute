use shared::{
    custom_type_name::CustomTypeName,
    wgsl_components::{
        WgslDerivedType, WgslInputArray, WgslOutputArray, WgslShaderModuleComponent, WgslType,
    },
};

use crate::{
    state::{self, ModuleTransformState},
    transformer::{
        custom_types::custom_type::{CustomType, CustomTypeKind},
        to_wgsl_syntax::convert_to_wgsl,
    },
};
use quote::quote;

pub fn divide_custom_types_by_category(state: &mut ModuleTransformState) {
    assert!(state.allowed_types.is_some());
    for custom_type in state.allowed_types.as_ref().unwrap().custom_types.iter() {
        match custom_type.kind {
            CustomTypeKind::GpuOnlyHelperType => state
                .result
                .helper_types
                .push(custom_type.clone().into_wgsl_type(state)),
            CustomTypeKind::InputArray => state.result.input_arrays.push(WgslInputArray {
                item_type: custom_type.clone().into_wgsl_type(state),
                array_type: wgsl_input_array_def_from_item_type(custom_type, state),
            }),
            CustomTypeKind::OutputArray => {
                state.result.output_arrays.push(WgslOutputArray {
                    item_type: custom_type.clone().into_wgsl_type(state),
                    array_type: wgsl_output_array_def_from_item_type(custom_type, state),
                    atomic_counter_type: None,
                });
            }
            CustomTypeKind::OutputVec => {
                state.result.output_arrays.push(WgslOutputArray {
                    item_type: custom_type.clone().into_wgsl_type(state),
                    array_type: wgsl_output_array_def_from_item_type(custom_type, state),
                    atomic_counter_type: Some(wgsl_atomic_counter_def_from_item_type(
                        custom_type,
                        state,
                    )),
                });
            }
            CustomTypeKind::Uniform => state
                .result
                .uniforms
                .push(custom_type.clone().into_wgsl_type(state)),
        }
    }
}

fn wgsl_input_array_def_from_item_type(
    item: &CustomType,
    state: &ModuleTransformState,
) -> WgslDerivedType {
    let rust_code = format!(
        "alias {} = array<{},{}>;",
        item.name.input_array(),
        item.name.name,
        item.name.input_array_length()
    );
    WgslDerivedType {
        name: item.name.input_array().to_string(),
        code: WgslShaderModuleComponent {
            rust_code,
            wgsl_code: convert_to_wgsl(quote!(rust_code), &state).to_string(),
        },
    }
}
fn wgsl_output_array_def_from_item_type(
    item: &CustomType,
    state: &ModuleTransformState,
) -> WgslDerivedType {
    let rust_code = format!(
        "alias {} = array<{},{}>;",
        item.name.output_array(),
        item.name.name,
        item.name.output_array_length()
    );
    WgslDerivedType {
        name: item.name.output_array().to_string(),
        code: WgslShaderModuleComponent {
            rust_code,
            wgsl_code: convert_to_wgsl(quote!(rust_code), &state).to_string(),
        },
    }
}

fn wgsl_atomic_counter_def_from_item_type(
    item: &CustomType,
    state: &ModuleTransformState,
) -> WgslDerivedType {
    let rust_code = format!("alias {} = atomic<u32>;", item.name.counter());
    WgslDerivedType {
        name: item.name.counter().to_string(),
        code: WgslShaderModuleComponent {
            rust_code,
            wgsl_code: convert_to_wgsl(quote!(rust_code), state).to_string(),
        },
    }
}
