use bevy::render::renderer::RenderDevice;
use bevy_gpu_compute_core::TypesSpec;
use wgpu::{BufferDescriptor, BufferUsages, util::BufferInitDescriptor};

use crate::task::{
    outputs::definitions::wgsl_counter::WgslCounter, task_commands::GpuTaskCommands,
    task_components::task::BevyGpuComputeTask,
};

pub fn update_output_buffers(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    let mut output_buffers = Vec::new();
    let mut output_staging_buffers = Vec::new();
    let mut output_count_buffers = Vec::new();
    let mut output_count_staging_buffers = Vec::new();
    // Collect all metadata first to release the immutable borrow
    let metadata: Vec<_> = task
        .spec
        .output_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .cloned()
        .collect();
    for (i, output_spec) in metadata.iter().enumerate() {
        if let Some(spec) = output_spec {
            let length = task.spec.output_array_lengths().get_by_name(spec.name());
            let output_size = spec.get_bytes() as u64 * length as u64;
            let output_buffer = render_device.create_buffer(&BufferDescriptor {
                label: Some(&format!("{:}-output-{:}", task.name(), i)),
                size: output_size,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            output_buffers.push(output_buffer);
            let output_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{:}-output-staging-{:}", task.name(), i)),
                size: output_size,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            output_staging_buffers.push(output_staging_buffer);
            if spec.get_include_count() {
                let counter_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some(&format!("{:}-output-counter-{:}", task.name(), i)),
                    contents: bytemuck::cast_slice(&[WgslCounter { count: 0 }]),
                    usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
                });
                output_count_buffers.push(counter_buffer);
                let counter_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("{:}-output-counter-staging-{:}", task.name(), i)),
                    size: std::mem::size_of::<WgslCounter>() as u64,
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                output_count_staging_buffers.push(counter_staging_buffer);
            }
        }
    }
    task.buffers.output = output_buffers;
    task.buffers.output_staging = output_staging_buffers;
    task.buffers.output_count = output_count_buffers;
    task.buffers.output_count_staging = output_count_staging_buffers;
}
