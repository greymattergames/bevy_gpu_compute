use bevy::{
    prelude::Res,
    render::renderer::{RenderDevice, RenderQueue},
};
use bytemuck::Pod;
use pollster::FutureExt;
use wgpu::Buffer;

use super::{output_data::OutputData, output_spec::OutputVectorTypesSpec};

pub fn get_raw_gpu_result_vec(
    render_device: &Res<RenderDevice>,
    render_queue: &Res<RenderQueue>,
    output_buffer: &Buffer,
    staging_buffer: &Buffer,
    total_byte_size: u64,
    // buffer_view_converter: Box<dyn Fn(&[u8]) -> Box<dyn Any + Send + Sync> + Send + Sync>,
) -> Option<Vec<u8>> {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(
        &output_buffer,
        0,
        &staging_buffer,
        0,
        // std::mem::size_of::<WgslCounter>() as u64,
        total_byte_size,
    );
    render_queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buffer.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    render_device.poll(wgpu::Maintain::Wait);

    let result: Option<Vec<u8>> = if receiver.block_on().unwrap().is_ok() {
        let data = slice.get_mapped_range();
        let transformed_data = &*data;
        let r = Some(transformed_data.to_vec());
        drop(data);
        staging_buffer.unmap();
        output_buffer.unmap();
        r
    } else {
        None
    };
    result
}

pub fn get_raw_gpu_result_counter<T: 'static + Pod>(
    render_device: &Res<RenderDevice>,
    render_queue: &Res<RenderQueue>,
    output_buffer: &Buffer,
    staging_buffer: &Buffer,
    total_byte_size: u64,
    // buffer_view_converter: Box<dyn Fn(&[u8]) -> Box<dyn Any + Send + Sync> + Send + Sync>,
) -> Option<T> {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(
        &output_buffer,
        0,
        &staging_buffer,
        0,
        // std::mem::size_of::<WgslCounter>() as u64,
        // std::mem::size_of::<WgslCounter>() as u64,
        total_byte_size,
    );
    render_queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buffer.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    render_device.poll(wgpu::Maintain::Wait);

    let result = if receiver.block_on().unwrap().is_ok() {
        let data = slice.get_mapped_range();
        let transformed_data = &*data;
        if transformed_data.len() != std::mem::size_of::<T>() {
            return None;
        }
        let result = Some(bytemuck::pod_read_unaligned(transformed_data));
        // let converter = BufferViewConverter::new(&*data);
        // let result = converter.get::<T>();
        drop(data);
        result
    } else {
        None
    };
    staging_buffer.unmap();
    output_buffer.unmap();
    result
}
