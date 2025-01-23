use bevy::{log::info, render::renderer::RenderDevice};
use bevy_gpu_compute_core::TypesSpec;
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::{task_commands::GpuTaskCommands, task_components::task::BevyGpuComputeTask};

pub fn update_input_buffers(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    task.buffers.input.clear();
    let mut new_buffers = Vec::new();
    for (i, spec) in task
        .spec
        .input_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
    {
        if let Some(s) = spec {
            let label = format!("{}-input-{}", task.name(), s.name().name());
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(&label),
                contents: task
                    .input_data
                    .as_ref()
                    .unwrap()
                    .input_bytes(s.name().name())
                    .unwrap(),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });
            info!(
                "Created input buffer for task {} with label {}",
                task.name(),
                label
            );
            continue;
        }
    }
    task.buffers.input = new_buffers;
}
