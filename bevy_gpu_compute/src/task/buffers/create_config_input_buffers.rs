use bevy::{
    log::{self, info},
    render::renderer::RenderDevice,
};
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::task_components::task::BevyGpuComputeTask;

pub fn update_config_input_buffers(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    log::info!("Creating config input buffers for task {}", task.name());
    task.buffers.config_input.clear();
    let mut new_buffers = Vec::new();
    for spec in task
        .spec
        .config_input_metadata_spec()
        .get_all_metadata()
        .iter()
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
                "Created config input buffer for task {} with label {}",
                task.name(),
                label
            );
            new_buffers.push(buffer);
            continue;
        }
    }
    task.buffers.config_input = new_buffers;
}
