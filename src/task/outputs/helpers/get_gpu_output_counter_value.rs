use bevy::{
    log,
    render::renderer::{RenderDevice, RenderQueue},
};
use pollster::FutureExt;
use wgpu::Buffer;

use crate::task::outputs::definitions::wgsl_counter::WgslCounter;

pub fn get_gpu_output_counter_value(
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    output_buffer: &Buffer,
    staging_buffer: &Buffer,
    total_byte_size: u64,
) -> Option<WgslCounter> {
    log::info!("Reading GPU output counter value");
    let mut encoder = render_device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, total_byte_size);
    render_queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buffer.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    render_device.poll(wgpu::Maintain::Wait);
    log::info!("Reading GPU output counter value - waiting for map to complete");
    let result = if receiver.block_on().unwrap().is_ok() {
        let data = slice.get_mapped_range();
        let transformed_data = &*data;
        if transformed_data.len() != std::mem::size_of::<WgslCounter>() {
            return None;
        }
        let result = Some(bytemuck::pod_read_unaligned(transformed_data));
        drop(data);
        result
    } else {
        None
    };
    log::info!("Reading GPU output counter value - map completed");
    staging_buffer.unmap();
    log::info!("Reading GPU output counter value - unmap staging completed");
    log::info!("Gpu counter result: {:?}", result);
    result
}
