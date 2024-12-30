use core::task;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
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

use crate::code::gpu_power_user::{
    iteration_space_dependent_resources::resources::MaxNumGpuOutputItemsPerOutputType,
    output_spec::{GpuAccBevyComputeTaskOutputSpec, GpuAccBevyComputeTaskOutputSpecs},
    resources::GpuAccBevy,
    wgsl_processable_types::WgslCollisionResult,
};

use super::{
    get_results_count_from_gpu::get_raw_gpu_result_vec,
    resources::{LatestResultsStore, OutputBuffers, OutputCountsFromGpu, OutputStagingBuffers},
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
            &GpuAccBevyComputeTaskOutputSpecs,
        ),
        With<GpuAccBevy>,
    >,
    mut task_mut: Query<(Entity, &mut LatestResultsStore), With<GpuAccBevy>>,
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
                output_specs
                    .specs
                    .iter()
                    .for_each(|(label, (item_bytes, type_id))| {
                        let out_buffer = output_buffers.0.get(label).unwrap();
                        let staging_buffer = output_staging_buffers.0.get(label).unwrap();
                        let result: Box<dyn Any + Send + Sync> = handle_buffer_type(
                            type_id,
                            render_device.clone(),
                            render_queue.clone(),
                            **out_buffer,
                            **staging_buffer,
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
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    output_buffer: Buffer,
    staging_buffer: Buffer,
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
                );
                return Box::new(result);
            }
        };
    }

    // Register known types
    handle_type!(WgslCollisionResult);
    handle_type!(u128);
    handle_type!(NonPodType);
    panic!("Unregistered type ID: {:?}", type_id);
}

struct NonPodType {
    data: String,
}
