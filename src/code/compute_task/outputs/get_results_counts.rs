use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{Query, Res, ResMut, With},
    render::renderer::{RenderDevice, RenderQueue},
};
use bytemuck::Pod;
use pollster::FutureExt;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::{Buffer, BufferView};

use crate::code::compute_task::{
    buffers::components::{OutputCountBuffers, OutputCountStagingBuffers},
    resources::{GpuAcceleratedBevy, TaskLabel},
    wgsl_processable_types::{WgslCollisionResult, WgslCounter},
};

use super::{
    get_raw_gpu_results::get_raw_gpu_result_single, misc_components::OutputCountsFromGpu,
    output_spec::OutputSpecs,
};

pub fn get_results_counts(
    mut tasks: Query<
        (
            &TaskLabel,
            &OutputSpecs,
            &OutputCountBuffers,
            &OutputCountStagingBuffers,
            &mut OutputCountsFromGpu,
        ),
        With<GpuAcceleratedBevy>,
    >,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                task,
                output_specs,
                count_buffer,
                count_staging_buffer,
                mut results_count_from_gpu,
            )| {
                get_results_counts_single_task(
                    output_specs,
                    &render_device,
                    &render_queue,
                    &count_buffer.0.get(&task.0).unwrap(),
                    &count_staging_buffer.0.get(&task.0).unwrap(),
                    &mut results_count_from_gpu,
                );
            },
        );
}

fn get_results_counts_single_task(
    output_specs: &OutputSpecs,
    render_device: &Res<RenderDevice>,
    render_queue: &Res<RenderQueue>,
    count_buffer: &Buffer,
    count_staging_buffer: &Buffer,
    results_count_from_gpu: &mut OutputCountsFromGpu,
) {
    let local_res_counts: Arc<Mutex<HashMap<String, Option<usize>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    output_specs.specs.par_iter().for_each(|(key, _)| {
        let count = get_results_counts_single_output_type(
            &render_device,
            &render_queue,
            count_buffer,
            count_staging_buffer,
        );
        local_res_counts
            .lock()
            .unwrap()
            .insert(key.to_string(), Some(count as usize));
    });
    results_count_from_gpu.0 = local_res_counts.lock().unwrap().clone();
}

fn get_results_counts_single_output_type(
    render_device: &Res<RenderDevice>,
    render_queue: &Res<RenderQueue>,
    count_buffer: &Buffer,
    count_staging_buffer: &Buffer,
) -> u32 {
    let count = get_raw_gpu_result_single::<WgslCounter>(
        &render_device,
        &render_queue,
        &count_buffer,
        &count_staging_buffer,
        std::mem::size_of::<WgslCounter>() as u64,
    );
    count.unwrap().count
}
