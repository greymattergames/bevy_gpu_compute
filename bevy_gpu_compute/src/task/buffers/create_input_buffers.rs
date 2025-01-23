use bevy::{log::info, render::renderer::RenderDevice};
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::task_components::task::BevyGpuComputeTask;

pub fn update_input_buffers(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    task.buffers.input.clear();
    let mut new_buffers = Vec::new();
    for spec in task
        .spec
        .input_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
    {
        if let Some(s) = spec {
            let label = format!("{}-input-{}", task.name(), s.name().name());
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(&label),
                contents: task
                    .input_data
                    .as_ref()
                    .unwrap()
                    .get_bytes(s.name().name())
                    .unwrap(),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });
            new_buffers.push(buffer);
            info!(
                "Created input buffer for task {} with label {}",
                task.name(),
                label
            );
        }
    }
    task.buffers.input = new_buffers;
}
