use bevy::{
    ecs::batching::BatchingStrategy,
    log::info,
    prelude::{EventReader, Query, Res},
    render::renderer::RenderDevice,
};
use bevy_gpu_compute_core::TypesSpec;
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::{
    inputs::config_type::config_input_metadata_spec::ConfigInputsMetadataSpec,
    task_commands::GpuTaskCommands,
    task_components::{task::BevyGpuComputeTask, task_name::TaskName},
    task_specification::task_specification::ComputeTaskSpecification,
};

use super::components::{ConfigInputBuffers, InputBuffers};

pub fn update_config_input_buffers(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    task.buffers.config_input.clear();
    let mut new_buffers = Vec::new();
    for (i, spec) in task
        .spec
        .config_input_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
    {
        if let Some(s) = spec {
            let label = format!("{}-input-{}", task.name(), s.name().name());
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(&label),
                contents: task
                    .config_input_data
                    .as_ref()
                    .unwrap()
                    .get_bytes(s.name().name())
                    .unwrap(),
                usage: BufferUsages::UNIFORM,
            });
            info!(
                "Created input buffer for task {} with label {}",
                task.name(),
                label
            );
            new_buffers.push(buffer);
            continue;
        }
    }
    task.buffers.config_input = new_buffers;
}
