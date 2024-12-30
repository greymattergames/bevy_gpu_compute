use std::any::Any;

use bevy::{
    prelude::{Res, ResMut},
    render::renderer::{RenderDevice, RenderQueue},
};
use bytemuck::Pod;
use pollster::FutureExt;
use wgpu::{Buffer, BufferView};

use crate::code::gpu_power_user::{
    output_spec::{BufferViewConverter, FromBytes},
    resources::CounterStagingBuffer,
    wgsl_processable_types::{WgslCollisionResult, WgslCounter},
};

use super::resources::{ResultsCountFromGpu, SingleBatchBuffers};

pub fn get_results_counts_from_gpu(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    buffers: Res<SingleBatchBuffers>,
    counter_staging_buffer: Res<CounterStagingBuffer>,
    mut results_count_from_gpu: ResMut<ResultsCountFromGpu>,
) {
    let count: Option<u32> =
        get_raw_gpu_result_single(render_device, render_queue, output_buffer, staging_buffer);
    results_count_from_gpu.0 = count.unwrap();
}

pub fn get_raw_gpu_result_vec<T: 'static + Pod>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    output_buffer: Buffer,
    staging_buffer: Buffer,
    // buffer_view_converter: Box<dyn Fn(&[u8]) -> Box<dyn Any + Send + Sync> + Send + Sync>,
) -> Option<Vec<T>> {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(
        &output_buffer,
        0,
        &staging_buffer,
        0,
        std::mem::size_of::<WgslCounter>() as u64,
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
        let result = converter.get_vec::<T>();
        drop(data);
        result
    } else {
        None
    };
    staging_buffer.unmap();
    output_buffer.unmap();
    result
}

pub fn get_raw_gpu_result_single<T: 'static + Pod>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    output_buffer: Buffer,
    staging_buffer: Buffer,
    // buffer_view_converter: Box<dyn Fn(&[u8]) -> Box<dyn Any + Send + Sync> + Send + Sync>,
) -> Option<T> {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(
        &output_buffer,
        0,
        &staging_buffer,
        0,
        std::mem::size_of::<WgslCounter>() as u64,
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
