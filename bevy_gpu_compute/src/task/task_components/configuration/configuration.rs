use std::collections::HashMap;

use bevy::{log, prelude::Component, render::renderer::RenderDevice, tasks::Task};
use bevy_gpu_compute_core::{
    MaxOutputLengths, TypesSpec,
    wgsl::shader_module::{
        complete_shader_module::WgslShaderModule, user_defined_portion::WgslShaderModuleUserPortion,
    },
};

use super::{iteration_space::IterationSpace, wgsl_code::WgslCode};
use crate::task::task_components::configuration::input_spec::InputSpec;
use crate::task::task_components::configuration::output_spec::OutputSpec;

/**
These all used to be separate components, but this limited the user api, for example the user could not update the iteration space and then retrieve the resulting correct GpuWorkgroupSpace/Sizes in the same frame, since these updates were handled in separate systems.
The size of this component should still be quite small, so the tradeoff of having a larger component for a better user api is worth it.
*/
#[derive(Default)]
pub struct TaskConfiguration {
    // Core configuration that defines the task
    shader: WgslCode,
    iteration_space: IterationSpace,

    // Input/Output specifications
    inputs: InputSpec,
    outputs: OutputSpec,
    version: u64,
}

impl TaskConfiguration {
    pub fn new(
        shader: WgslCode,
        iteration_space: IterationSpace,
        inputs: InputSpec,
        outputs: OutputSpec,
    ) -> Self {
        TaskConfiguration {
            shader,
            iteration_space,
            inputs,
            outputs,
            version: 0,
        }
    }

    pub fn shader(&self) -> &WgslCode {
        &self.shader
    }

    pub fn iteration_space(&self) -> &IterationSpace {
        &self.iteration_space
    }

    pub fn inputs(&self) -> &InputSpec {
        &self.inputs
    }

    pub fn outputs(&self) -> &OutputSpec {
        &self.outputs
    }
    pub fn version(&self) -> u64 {
        self.version
    }
    /// make sure you are actually changing the max lengths when you call this, otherwise the config version will be updated and cause unecessary recalculation of pipeline consts
    pub fn outputs_mut(&mut self) -> &mut OutputSpec {
        self.version += 1;
        &mut self.outputs
    }
    /// ensure that the runtime state has been properly updated whenever we change the iteration space
    pub fn _internal_set_iteration_space(&mut self, new_iteration_space: IterationSpace) {
        self.version += 1;
        self.iteration_space = new_iteration_space;
    }
}
