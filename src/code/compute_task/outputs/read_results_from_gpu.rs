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
    ecs::batching::BatchingStrategy,
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
    misc_components::OutputCountsFromGpu, output_spec::OutputSpecs,
};

/**
 *   We put this all into a single system instead of passing with resources because we cannot pass the buffer slice around without lifetimes
 * The way the WGSL code works we can guarantee no duplicate collision detections WITHIN THE SAME FRAME due to entity ordering (as long as the batcher doesn't mess up the order when splitting up the data), but a collision detected as (entity1, entity2) in one frame may be detected as (entity2, entity1) in the next frame.
 * */
pub fn read_results_from_gpu(
    task: Query<
        (
            Entity,
            &OutputBuffers,
            &OutputStagingBuffers,
            &OutputCountsFromGpu,
            &MaxNumGpuOutputItemsPerOutputType,
            &OutputSpecs,
        ),
        With<GpuAcceleratedBevy>,
    >,
    mut task_mut: Query<(Entity, &mut LatestResultsStore), With<GpuAcceleratedBevy>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let results_per_task: Arc<Mutex<HashMap<Entity, Vec<(String, Box<dyn Any + Send + Sync>)>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    task.par_iter()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                entity,
                output_buffers,
                output_staging_buffers,
                output_counts,
                max_outputs,
                output_specs,
            )| {
                output_specs.specs.iter().for_each(|(label, spec)| {
                    let out_buffer = output_buffers.0.get(label).unwrap();
                    let staging_buffer = output_staging_buffers.0.get(label).unwrap();
                    let total_byte_size = min(
                        if let Some(Some(c)) = output_counts.0.get(label) {
                            c * spec.item_bytes
                        } else {
                            usize::MAX
                        },
                        max_outputs.get(label) * spec.item_bytes,
                    );

                    let result: Box<dyn Any + Send + Sync> = handle_buffer_type(
                        spec.type_id,
                        &render_device,
                        &render_queue,
                        out_buffer,
                        staging_buffer,
                        total_byte_size,
                    );

                    let mut results_per_task = results_per_task.lock().unwrap();
                    if let Some(task_results) = results_per_task.get_mut(&entity) {
                        task_results.push((label.clone(), result));
                    } else {
                        results_per_task.insert(entity, vec![(label.clone(), result)]);
                    }
                });
            },
        );
    for mut task in task_mut.iter_mut() {
        let mut results_per_task = results_per_task.lock().unwrap();
        if let Some(results) = results_per_task.remove(&task.0) {
            // turn vec into hashmap
            let map: HashMap<String, Box<dyn std::any::Any + Send + Sync>> =
                results.into_iter().collect();
            task.1.results = map;
        }
    }
}

// Helper function to handle type dispatch
fn handle_buffer_type(
    type_id: TypeId,
    render_device: &Res<RenderDevice>,
    render_queue: &Res<RenderQueue>,
    output_buffer: &Buffer,
    staging_buffer: &Buffer,
    total_byte_size: usize,
) -> Box<dyn Any + Send + Sync> {
    // You'll need to maintain a registry of types and their handlers
    // Here's a macro-based approach:
    macro_rules! handle_type {
        ($t:ty) => {
            if type_id == TypeId::of::<Vec<$t>>() {
                let result = get_raw_gpu_result_vec::<$t>(
                    render_device,
                    render_queue,
                    output_buffer,
                    staging_buffer,
                    total_byte_size as u64,
                );
                return Box::new(result);
            }
        };
    }

    // Register known types
    handle_type!(WgslCollisionResult);
    handle_type!(u128);
    panic!("Unregistered type ID: {:?}", type_id);
}

struct NonPodType {
    data: String,
}
