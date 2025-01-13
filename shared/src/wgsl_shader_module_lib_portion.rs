use crate::wgsl_components::{
    WORKGROUP_SIZE_X_VAR_NAME, WORKGROUP_SIZE_Y_VAR_NAME, WORKGROUP_SIZE_Z_VAR_NAME,
    WgpuShaderType, WgslShaderModuleComponent,
};

use super::{
    wgsl_components::{
        WgslConstAssignment, WgslFunction, WgslShaderModuleUserPortion, WgslType,
        WgslWorkgroupDeclaration,
    },
    wgsl_wgpu_binding::WgslWgpuBinding,
};

pub struct WgslShaderModuleLibraryPortion {
    // generate these based on inputs and outputs
    pub pipeline_consts: Vec<WgslConstAssignment>,
    /// currently unused
    pub uniforms: Vec<WgslType>,
    /// currently unused
    pub helper_functions: Vec<WgslFunction>,
    /// static, generate automatically from the user portion
    pub bindings: Vec<WgslWgpuBinding>,
    /// static, workgroup sizes changed via pipeline consts
    pub workgroups_declaration: WgslWorkgroupDeclaration,
}

impl From<&WgslShaderModuleUserPortion> for WgslShaderModuleLibraryPortion {
    fn from(user_portion: &WgslShaderModuleUserPortion) -> Self {
        let mut pipeline_consts = vec![
            WgslConstAssignment::new(WORKGROUP_SIZE_X_VAR_NAME, "u32", "64"),
            WgslConstAssignment::new(WORKGROUP_SIZE_Y_VAR_NAME, "u32", "1"),
            WgslConstAssignment::new(WORKGROUP_SIZE_Z_VAR_NAME, "u32", "1"),
        ];
        let mut binding_counter = 0;
        let mut bindings = Vec::new();
        user_portion.uniforms.iter().for_each(|u| {
            bindings.push(WgslWgpuBinding::uniform(
                0,
                binding_counter,
                u.name.uniform(),
                u.name.name.clone(),
            ));
            binding_counter += 1;
        });
        user_portion.input_arrays.iter().for_each(|a| {
            pipeline_consts.push(WgslConstAssignment::new(
                &a.item_type.name.input_array_length(),
                "u32",
                "1",
            ));
            bindings.push(WgslWgpuBinding::input_array(
                0,
                binding_counter,
                a.item_type.name.input_array(),
                a.array_type.name.clone(),
            ));
            binding_counter += 1;
        });
        user_portion.output_arrays.iter().for_each(|a| {
            pipeline_consts.push(WgslConstAssignment::new(
                &a.item_type.name.output_array_length(),
                "u32",
                "1",
            ));
            let output_array = WgslWgpuBinding::output_array(
                0,
                binding_counter,
                a.item_type.name.output_array(),
                a.array_type.name.clone(),
            );
            bindings.push(output_array.clone());
            binding_counter += 1;
            if let Some(atomic_counter_name) = &a.atomic_counter_name {
                bindings.push(WgslWgpuBinding::counter(binding_counter, &a, &output_array));
                binding_counter += 1;
            }
        });
        WgslShaderModuleLibraryPortion {
            pipeline_consts,
            uniforms: Vec::new(),
            helper_functions: Vec::new(),
            bindings: bindings,
            workgroups_declaration: WgslWorkgroupDeclaration {
                shader_type: WgpuShaderType::Compute,
            },
        }
    }
}
