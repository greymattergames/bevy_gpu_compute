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
    component::TaskName,
    resources::GpuAcceleratedBevy,
    wgsl_processable_types::{WgslCollisionResult, WgslCounter},
};

use super::{
    get_raw_gpu_results::get_raw_gpu_result_counter, misc_components::OutputCountsFromGpu,
    output_metadata_spec::OutputVectorMetadataSpec,
};

pub fn get_results_counts_from_gpu(
    mut tasks: Query<
        (
            &OutputVectorMetadataSpec,
            &OutputCountBuffers,
            &OutputCountStagingBuffers,
            &mut OutputCountsFromGpu,
        ),
        With<GpuAcceleratedBevy>,
    >,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    //todo, when should this run?
    //after dispatch before main results read
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(output_specs, count_buffers, count_staging_buffers, mut results_count_from_gpu)| {
                get_results_counts_single_task(
                    output_specs,
                    &render_device,
                    &render_queue,
                    &count_buffers,
                    &count_staging_buffers,
                    &mut results_count_from_gpu,
                );
            },
        );
}

fn get_results_counts_single_task(
    output_specs: &OutputVectorMetadataSpec,
    render_device: &Res<RenderDevice>,
    render_queue: &Res<RenderQueue>,
    count_buffers: &OutputCountBuffers,
    count_staging_buffers: &OutputCountStagingBuffers,
    results_count_from_gpu: &mut OutputCountsFromGpu,
) {
    let local_res_counts: Arc<Mutex<Vec<Option<usize>>>> = Arc::new(Mutex::new(Vec::new()));
    output_specs
        .get_all_metadata()
        .iter()
        .enumerate()
        .for_each(|(i, spec)| {
            if let Some(s) = spec {
                if s.get_include_count() {
                    let count = get_results_counts_single_output_type(
                        &render_device,
                        &render_queue,
                        &count_buffers.0[i],
                        &count_staging_buffers.0[i],
                    );
                    local_res_counts.lock().unwrap().push(Some(count as usize));
                } else {
                    local_res_counts.lock().unwrap().push(None);
                }
            } else {
                local_res_counts.lock().unwrap().push(None);
            }
        });
    results_count_from_gpu.0 = local_res_counts.lock().unwrap().clone();
}

fn get_results_counts_single_output_type(
    render_device: &Res<RenderDevice>,
    render_queue: &Res<RenderQueue>,
    count_buffer: &Buffer,
    count_staging_buffer: &Buffer,
) -> u32 {
    let count = get_raw_gpu_result_counter::<WgslCounter>(
        &render_device,
        &render_queue,
        &count_buffer,
        &count_staging_buffer,
        std::mem::size_of::<WgslCounter>() as u64,
    );
    count.unwrap().count
}
