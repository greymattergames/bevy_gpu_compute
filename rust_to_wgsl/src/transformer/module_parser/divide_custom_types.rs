use proc_macro_error::abort;
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
        to_wgsl_syntax::convert_file_to_wgsl,
    },
};
use quote::quote;

pub fn divide_custom_types_by_category(state: &mut ModuleTransformState) {
    let allowed_types = if let Some(ct) = state.allowed_types.clone() {
        ct
    } else {
        abort!(
            state.rust_module.ident.span(),
            "Allowed types must be set before dividing custom types"
        );
    };
    for custom_type in allowed_types.custom_types.iter() {
        match custom_type.kind {
            CustomTypeKind::GpuOnlyHelperType => state
                .result
                .helper_types
                .push(custom_type.clone().into_wgsl_type(&state)),
            CustomTypeKind::InputArray => {
                state
                    .allowed_types
                    .as_mut()
                    .unwrap()
                    .add_user_type(CustomType::new(
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
                state
                    .allowed_types
                    .as_mut()
                    .unwrap()
                    .add_user_type(CustomType::new(
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
                state
                    .allowed_types
                    .as_mut()
                    .unwrap()
                    .add_user_type(CustomType::new(
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
