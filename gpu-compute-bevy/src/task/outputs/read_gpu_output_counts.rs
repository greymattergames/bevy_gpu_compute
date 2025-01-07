use std::sync::{Arc, Mutex};

use bevy::{
    ecs::batching::BatchingStrategy,
    log,
    prelude::{Query, Res},
    render::renderer::{RenderDevice, RenderQueue},
};
use wgpu::Buffer;

use crate::task::{
    buffers::components::{OutputCountBuffers, OutputCountStagingBuffers},
    task_specification::task_specification::TaskUserSpecification,
};

use super::{
    definitions::{
        gpu_output_counts::GpuOutputCounts, output_vector_metadata_spec::OutputVectorsMetadataSpec,
        wgsl_counter::WgslCounter,
    },
    helpers::get_gpu_output_counter_value::get_gpu_output_counter_value,
};

pub fn read_gpu_output_counts(
    mut tasks: Query<(
        &TaskUserSpecification,
        &OutputCountBuffers,
        &OutputCountStagingBuffers,
        &mut GpuOutputCounts,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    log::info!("Reading GPU output counts");
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(task_spec, count_buffers, count_staging_buffers, mut results_count_from_gpu)| {
                read_gpu_output_counts_single_task(
                    task_spec.output_vectors_metadata_spec(),
                    &render_device,
                    &render_queue,
                    &count_buffers,
                    &count_staging_buffers,
                    &mut results_count_from_gpu,
                );
            },
        );
}

fn read_gpu_output_counts_single_task(
    output_specs: &OutputVectorsMetadataSpec,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    count_buffers: &OutputCountBuffers,
    count_staging_buffers: &OutputCountStagingBuffers,
    results_count_from_gpu: &mut GpuOutputCounts,
) {
    let local_res_counts: Arc<Mutex<Vec<Option<usize>>>> = Arc::new(Mutex::new(Vec::new()));
    output_specs
        .get_all_metadata()
        .iter()
        .enumerate()
        .for_each(|(i, spec)| {
            if let Some(s) = spec {
                if s.get_include_count() {
                    log::info!("Reading count for output {}", i);
                    let count = read_gpu_output_counts_single_output_type(
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

fn read_gpu_output_counts_single_output_type(
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    count_buffer: &Buffer,
    count_staging_buffer: &Buffer,
) -> u32 {
    let count = get_gpu_output_counter_value(
        &render_device,
        &render_queue,
        &count_buffer,
        &count_staging_buffer,
        std::mem::size_of::<WgslCounter>() as u64,
    );
    count.unwrap().count
}
