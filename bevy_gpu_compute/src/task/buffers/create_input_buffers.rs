use bevy::{log::info, render::renderer::RenderDevice};
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::task::BevyGpuComputeTask;

pub fn update_input_buffers(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    task.buffers_mut().input.clear();
    let mut new_buffers = Vec::new();
    for spec in task
        .configuration()
        .inputs()
        .arrays()
        .get_all_metadata()
        .iter()
    {
        if let Some(s) = spec {
            let label = format!("{}-input-{}", task.name(), s.name().name());
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(&label),
                contents: task
                    .current_data()
                    .input()
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
    task.buffers_mut().input = new_buffers;
}
