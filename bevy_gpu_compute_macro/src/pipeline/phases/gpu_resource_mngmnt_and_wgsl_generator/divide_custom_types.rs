use bevy_gpu_compute_core::wgsl::{
    shader_module::user_defined_portion::WgslShaderModuleUserPortion,
    shader_sections::{WgslInputArray, WgslOutputArray},
};

use crate::pipeline::phases::custom_type_collector::custom_type::{CustomType, CustomTypeKind};
use quote::quote;

pub fn generate_helper_types_inputs_and_outputs_for_wgsl_module_def(
    custom_types: &Vec<CustomType>,
    wgsl_module_def: &mut WgslShaderModuleUserPortion,
) -> Vec<CustomType> {
    let mut additional_custom_types: Vec<CustomType> = Vec::new();
    for custom_type in custom_types.iter() {
        match custom_type.kind {
            CustomTypeKind::GpuOnlyHelperType => {
                wgsl_module_def
                    .helper_types
                    .push(custom_type.clone().into_wgsl_type(custom_types));
            }
            CustomTypeKind::InputArray => {
                additional_custom_types.push(CustomType::new(
                    &custom_type.name.input_array_length(),
                    CustomTypeKind::ArrayLengthVariable,
                    quote!(),
                ));
                wgsl_module_def.input_arrays.push(WgslInputArray {
                    item_type: custom_type.clone().into_wgsl_type(custom_types),
                });
            }
            CustomTypeKind::OutputArray => {
                additional_custom_types.push(CustomType::new(
                    &custom_type.name.output_array_length(),
                    CustomTypeKind::ArrayLengthVariable,
                    quote!(),
                ));
                wgsl_module_def.output_arrays.push(WgslOutputArray {
                    item_type: custom_type.clone().into_wgsl_type(custom_types),
                    atomic_counter_name: None,
                });
            }
            CustomTypeKind::OutputVec => {
                additional_custom_types.push(CustomType::new(
                    &custom_type.name.output_array_length(),
                    CustomTypeKind::ArrayLengthVariable,
                    quote!(),
                ));
                wgsl_module_def.output_arrays.push(WgslOutputArray {
                    item_type: custom_type.clone().into_wgsl_type(custom_types),
                    atomic_counter_name: Some(custom_type.name.counter().to_string()),
                });
            }
            CustomTypeKind::Uniform => {
                wgsl_module_def
                    .uniforms
                    .push(custom_type.clone().into_wgsl_type(custom_types));
            }
            CustomTypeKind::ArrayLengthVariable => {
                // do nothing
            }
        }
    }

    // add the additional custom types to the custom types list
    let mut out = custom_types.clone();
    out.extend(additional_custom_types);
    out
}
