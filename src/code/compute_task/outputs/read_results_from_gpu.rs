use core::task;
use std::{
    any::{Any, TypeId},
    cmp::min,
    collections::HashMap,
    f64::MIN,
    process::Output,
    sync::{Arc, Mutex},
};

use bevy::{
    asset::UnknownTyped,
    ecs::{batching::BatchingStrategy, query::QueryData},
    prelude::{Entity, Query, Res, ResMut, With},
    render::renderer::{RenderDevice, RenderQueue},
};
use bytemuck::Pod;
use pollster::FutureExt;
use wgpu::Buffer;

use crate::code::compute_task::{
    buffers::components::{OutputBuffers, OutputStagingBuffers},
    iteration_space_dependent_resources::max_num_outputs_per_type::MaxNumGpuOutputItemsPerOutputType,
    resources::GpuAcceleratedBevy,
    wgsl_processable_types::WgslCollisionResult,
};

use super::{
    get_raw_gpu_results::get_raw_gpu_result_vec, latest_results_store::LatestResultsStore,
    misc_components::OutputCountsFromGpu, output_data::OutputData,
    output_spec::OutputVectorTypesSpec,
};

/**
 *   We put this all into a single system instead of passing with resources because we cannot pass the buffer slice around without lifetimes
 * The way the WGSL code works we can guarantee no duplicate collision detections WITHIN THE SAME FRAME due to entity ordering (as long as the batcher doesn't mess up the order when splitting up the data), but a collision detected as (entity1, entity2) in one frame may be detected as (entity2, entity1) in the next frame.
 * */
pub fn read_results_from_gpu<O: OutputVectorTypesSpec + 'static + Send + Sync>(
    mut task: Query<
        (
            Entity,
            &OutputBuffers,
            &OutputStagingBuffers,
            &OutputCountsFromGpu,
            &MaxNumGpuOutputItemsPerOutputType,
            &mut OutputData<O>,
        ),
        With<GpuAcceleratedBevy>,
    >,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let results_per_task: Arc<Mutex<HashMap<Entity, Vec<(String, Box<dyn Any + Send + Sync>)>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    task.par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                entity,
                output_buffers,
                output_staging_buffers,
                output_counts,
                max_outputs,
                mut output_data,
            )| {
                OutputData::<O>::get_all_metadata()
                    .iter()
                    .enumerate()
                    .for_each(|(i, metadata)| {
                        if let Some(m) = metadata {
                            let out_buffer = output_buffers.0.get(i).unwrap();
                            let staging_buffer = output_staging_buffers.0.get(i).unwrap();
                            let total_byte_size = min(
                                if let Some(Some(c)) = output_counts.0.get(i) {
                                    c * m.bytes
                                } else {
                                    usize::MAX
                                },
                                max_outputs.get(i) * m.bytes,
                            );
                            get_raw_gpu_result_vec::<O>(
                                i,
                                &mut output_data,
                                &render_device,
                                &render_queue,
                                &out_buffer,
                                staging_buffer,
                                total_byte_size as u64,
                            );
                        }
                    });
            },
        );
}
