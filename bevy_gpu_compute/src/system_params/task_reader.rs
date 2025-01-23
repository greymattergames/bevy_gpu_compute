use std::collections::HashMap;

use bevy::{
    ecs::system::SystemParam,
    prelude::{Commands, Entity, Query, Res, Resource},
    render::renderer::{RenderDevice, RenderQueue},
};
use bevy_gpu_compute_core::{
    OutputDataBuilderTrait, TypesSpec,
    wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion,
};

use crate::{
    prelude::ComputeTaskSpecification,
    ram_limit::RamLimit,
    task::{
        buffers::{
            create_config_input_buffers::update_config_input_buffers,
            create_input_buffers::update_input_buffers,
            create_output_buffers::update_output_buffers,
        },
        compute_pipeline::update_on_pipeline_const_change::update_compute_pipeline,
        dispatch::{create_bind_group::create_bind_group, dispatch_to_gpu::dispatch_to_gpu},
        outputs::{
            read_gpu_output_counts::read_gpu_output_counts, read_gpu_task_outputs::read_gpu_outputs,
        },
        task_commands::{GpuTaskCommand, GpuTaskCommands},
        task_components::task::BevyGpuComputeTask,
        verify_enough_memory::verify_have_enough_memory,
    },
};

#[derive(SystemParam)]

pub struct GpuTaskReader<'w, 's> {
    tasks: Query<'w, 's, &'static mut BevyGpuComputeTask>,
}

impl<'w, 's> GpuTaskReader<'w, 's> {
    /// the latest result is cleared after this call, you cannot retrieve it a second time
    pub fn latest_results<OutputDataBuilder: OutputDataBuilderTrait>(
        &mut self,
        name: &str,
    ) -> Result<OutputDataBuilder, String> {
        let mut task = self
            .tasks
            .iter_mut()
            .find(|task| task.name() == name)
            .expect("Task not found");
        let result = if let Some(d) = &task.output_data {
            Ok(OutputDataBuilder::from(d))
        } else {
            Err("No output data found".into())
        };
        task.output_data = None;
        result
    }
}
