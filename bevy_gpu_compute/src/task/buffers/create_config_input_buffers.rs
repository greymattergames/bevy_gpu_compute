use bevy::{
    log::{self},
    render::renderer::RenderDevice,
};
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::lib::BevyGpuComputeTask;

pub fn update_config_input_buffers(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    log::trace!("Creating config input buffers for task {}", task.name());
    task.buffers_mut().config.clear();
    let mut new_buffers = Vec::new();
    for s in task.configuration().inputs().configs().iter() {
        let label = format!("{}-input-{}", task.name(), s.name.name());
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some(&label),
            contents: task
                .current_data()
                .config_input()
                .as_ref()
                .unwrap()
                .get_bytes(s.name.name())
                .unwrap(),
            usage: BufferUsages::UNIFORM,
        });
        log::trace!(
            "Created config input buffer for task {} with label {}",
            task.name(),
            label
        );
        new_buffers.push(buffer);
        continue;
    }
    task.buffers_mut().config = new_buffers;
}
