use bevy_gpu_compute_core::wgsl::shader_sections::{WgslInputArray, WgslOutputArray};
use proc_macro_error::abort;

use crate::{
    state::ModuleTransformState,
    transformer::custom_types::custom_type::{CustomType, CustomTypeKind},
};
use quote::quote;

pub fn divide_custom_types_by_category(state: &mut ModuleTransformState) {
    let custom_types = if let Some(ct) = state.custom_types.clone() {
        ct
    } else {
        abort!(
            state.rust_module.ident.span(),
            "Allowed types must be set before dividing custom types"
        );
    };
    for custom_type in custom_types.iter() {
        match custom_type.kind {
            CustomTypeKind::GpuOnlyHelperType => state
                .result
                .helper_types
                .push(custom_type.clone().into_wgsl_type(&state)),
            CustomTypeKind::InputArray => {
                state.custom_types.as_mut().unwrap().push(CustomType::new(
                    &custom_type.name.input_array_length(),
                    CustomTypeKind::ArrayLengthVariable,
                    quote!(),
                ));
                println!("HERE");
                state.result.input_arrays.push(WgslInputArray {
                    item_type: custom_type.clone().into_wgsl_type(&state),
                });
            }
            CustomTypeKind::OutputArray => {
                state.custom_types.as_mut().unwrap().push(CustomType::new(
                    &custom_type.name.output_array_length(),
                    CustomTypeKind::ArrayLengthVariable,
                    quote!(),
                ));
                state.result.output_arrays.push(WgslOutputArray {
                    item_type: custom_type.clone().into_wgsl_type(&state),
                    atomic_counter_name: None,
                });
            }
            CustomTypeKind::OutputVec => {
                state.custom_types.as_mut().unwrap().push(CustomType::new(
                    &custom_type.name.output_array_length(),
                    CustomTypeKind::ArrayLengthVariable,
                    quote!(),
                ));
                state.result.output_arrays.push(WgslOutputArray {
                    item_type: custom_type.clone().into_wgsl_type(state),
                    atomic_counter_name: Some(custom_type.name.counter().to_string()),
                });
            }
            CustomTypeKind::Uniform => state
                .result
                .uniforms
                .push(custom_type.clone().into_wgsl_type(state)),
            CustomTypeKind::ArrayLengthVariable => {
                // do nothing
            }
        }
    }
}
