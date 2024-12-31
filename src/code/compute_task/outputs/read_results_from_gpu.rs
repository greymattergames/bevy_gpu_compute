use core::{panic, task};
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
    prelude::{Commands, Entity, EventWriter, Query, Res, ResMut, With},
    render::renderer::{RenderDevice, RenderQueue},
};
use bytemuck::Pod;
use pollster::FutureExt;
use wgpu::Buffer;

use crate::code::{
    compute_task::{
        buffers::components::{OutputBuffers, OutputStagingBuffers},
        component::TaskRunId,
        events::GpuComputeTaskSuccessEvent,
        iteration_space_dependent_components::max_output_vector_lengths::MaxOutputVectorLengths,
        resources::GpuAcceleratedBevy,
        wgsl_processable_types::WgslCollisionResult,
    },
    manager_resource::GpuCompute,
};

use super::{
    get_raw_gpu_results::get_raw_gpu_result_vec,
    misc_components::OutputCountsFromGpu,
    output_data::{OutputData, TypeErasedOutputData},
    output_metadata_spec::OutputVectorMetadataSpec,
    output_spec::OutputVectorTypesSpec,
};

/**
 *   We put this all into a single system instead of passing with resources because we cannot pass the buffer slice around without lifetimes
 * The way the WGSL code works we can guarantee no duplicate collision detections WITHIN THE SAME FRAME due to entity ordering (as long as the batcher doesn't mess up the order when splitting up the data), but a collision detected as (entity1, entity2) in one frame may be detected as (entity2, entity1) in the next frame.
 * */
pub fn read_results_from_gpu(
    mut task: Query<
        (
            Entity,
            &TaskRunId,
            &OutputBuffers,
            &OutputStagingBuffers,
            &OutputCountsFromGpu,
            &MaxOutputVectorLengths,
            &OutputVectorMetadataSpec,
            &mut TypeErasedOutputData,
        ),
        With<GpuAcceleratedBevy>,
    >,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut commands: Commands,
    mut success_event_writer: EventWriter<GpuComputeTaskSuccessEvent>,
) {
    let events: Arc<Mutex<Vec<GpuComputeTaskSuccessEvent>>> = Arc::new(Mutex::new(Vec::new()));
    task.par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                entity,
                run_id,
                output_buffers,
                output_staging_buffers,
                output_counts,
                max_outputs,
                output_spec,
                mut out_data,
            )| {
                let mut type_erased_output = TypeErasedOutputData::empty();

                output_spec
                    .get_all_metadata()
                    .iter()
                    .enumerate()
                    .for_each(|(i, metadata)| {
                        if let Some(m) = metadata {
                            let out_buffer = output_buffers.0.get(i).unwrap();
                            let staging_buffer = output_staging_buffers.0.get(i).unwrap();
                            let total_byte_size = min(
                                if let Some(Some(c)) = output_counts.0.get(i) {
                                    c * m.get_bytes()
                                } else {
                                    usize::MAX
                                },
                                max_outputs.get(i) * m.get_bytes(),
                            );

                            let raw_bytes = get_raw_gpu_result_vec(
                                &render_device,
                                &render_queue,
                                &out_buffer,
                                staging_buffer,
                                total_byte_size as u64,
                            );
                            if let Some(raw_bytes) = raw_bytes {
                                type_erased_output.set_output_from_bytes(i, raw_bytes);
                            } else {
                                panic!("Failed to read output from GPU");
                            }
                        }
                    });
                *out_data = type_erased_output;
                events
                    .lock()
                    .unwrap()
                    .push(GpuComputeTaskSuccessEvent { id: run_id.0 });
            },
        );
    let events = events.lock().unwrap();
    success_event_writer.send_batch(events.into());
}
