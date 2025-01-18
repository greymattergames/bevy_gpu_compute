use core::panic;
use std::{
    cmp::min,
    sync::{Arc, Mutex},
};

use bevy::{
    ecs::batching::BatchingStrategy,
    log,
    prelude::{Commands, Entity, EventWriter, Query, Res},
    render::renderer::{RenderDevice, RenderQueue},
};

use crate::task::{
    buffers::components::{OutputBuffers, OutputStagingBuffers},
    events::GpuComputeTaskSuccessEvent,
    task_components::task_run_id::TaskRunId,
    task_specification::task_specification::ComputeTaskSpecification,
};

use super::{
    definitions::{
        gpu_output_counts::GpuOutputCounts, output_vector_metadata_spec::OutputVectorsMetadataSpec,
        type_erased_output_data::TypeErasedOutputData,
    },
    helpers::get_gpu_output_as_bytes_vec::get_gpu_output_as_bytes_vec,
};

/**
 * We put this all into a single system because we cannot pass the buffer slice around easily.
 * */
pub fn read_gpu_task_outputs(
    mut task: Query<(
        Entity,
        &TaskRunId,
        &OutputBuffers,
        &OutputStagingBuffers,
        &GpuOutputCounts,
        &ComputeTaskSpecification,
        &mut TypeErasedOutputData,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut commands: Commands,
    mut success_event_writer: EventWriter<GpuComputeTaskSuccessEvent>,
) {
    let run_ids_successfuls: Arc<Mutex<Vec<u128>>> = Arc::new(Mutex::new(Vec::new()));
    task.par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                entity,
                run_id,
                output_buffers,
                output_staging_buffers,
                output_counts,
                task_spec,
                mut out_data,
            )| {
                let mut type_erased_output = TypeErasedOutputData::empty();

                task_spec
                    .output_vectors_metadata_spec()
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
                                task_spec.output_array_lengths().get_by_name(m.name())
                                    * m.get_bytes(),
                            );

                            let raw_bytes = get_gpu_output_as_bytes_vec(
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
                log::info!("Read output for task {}", run_id.0);
                *out_data = type_erased_output;
                run_ids_successfuls.lock().unwrap().push(run_id.0);
            },
        );
    // map run ids into events
    let events: Vec<GpuComputeTaskSuccessEvent> = run_ids_successfuls
        .lock()
        .unwrap()
        .iter()
        .map(|id| GpuComputeTaskSuccessEvent { id: *id })
        .collect();
    success_event_writer.send_batch(events);
}
