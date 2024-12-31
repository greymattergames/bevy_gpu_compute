use bevy::{
    prelude::Res,
    render::renderer::{RenderDevice, RenderQueue},
};
use bytemuck::Pod;
use pollster::FutureExt;
use wgpu::Buffer;

use crate::code::compute_task::wgsl_processable_types::WgslCounter;

use super::{
    buffer_view_converter::BufferViewConverter, output_data::OutputData,
    output_spec::OutputVectorTypesSpec,
};

pub fn get_raw_gpu_result_vec<O: OutputVectorTypesSpec>(
    output_vector_index: usize,
    mut output_data: &mut OutputData<O>,
    render_device: &Res<RenderDevice>,
    render_queue: &Res<RenderQueue>,
    output_buffer: &Buffer,
    staging_buffer: &Buffer,
    total_byte_size: u64,
    // buffer_view_converter: Box<dyn Fn(&[u8]) -> Box<dyn Any + Send + Sync> + Send + Sync>,
) {
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

    if receiver.block_on().unwrap().is_ok() {
        let data = slice.get_mapped_range();
        let data2: &[u8] = &data;
        if output_vector_index == 0 {
            output_data.set_output1_from_bytes(data2);
        } else if output_vector_index == 1 {
            output_data.set_output2_from_bytes(data2);
        } else if output_vector_index == 2 {
            output_data.set_output3_from_bytes(data2);
        } else if output_vector_index == 3 {
            output_data.set_output4_from_bytes(data2);
        } else if output_vector_index == 4 {
            output_data.set_output5_from_bytes(data2);
        } else if output_vector_index == 5 {
            output_data.set_output6_from_bytes(data2);
        }
        drop(data);
        staging_buffer.unmap();
        output_buffer.unmap();
    }
}

pub fn get_raw_gpu_result_single<T: 'static + Pod>(
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
        let converter = BufferViewConverter::new(&*data);
        let result = converter.get::<T>();
        drop(data);
        result
    } else {
        None
    };
    staging_buffer.unmap();
    output_buffer.unmap();
    result
}
