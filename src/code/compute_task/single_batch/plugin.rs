use bevy::{
    app::{App, Plugin, Startup},
    prelude::{Commands, IntoSystemConfigs, Schedule, SystemSet},
};

use crate::code::compute_task::{
    custom_schedule::BatchedCollisionDetectionSchedule,
    iteration_space_dependent_resources::batch_size_dependent_resources::{
        pipeline::update::update_pipeline, update_wgsl_consts::update_wgsl_consts,
    },
};

use super::{
    convert_collidables_to_wgsl_types::convert_collidables_to_wgsl_types,
    create_bind_group::create_bind_group,
    create_buffers::create_buffers,
    dispatch_to_gpu::dispatch_to_gpu,
    finish_batch::finish_batch,
    get_results_count_from_gpu::get_results_count_from_gpu,
    initialize_batch::initialize_batch,
    read_results_from_gpu::read_results_from_gpu,
    resources::{
        BindGroup, CollidablesBatch, ResultsCountFromGpu, SingleBatchBuffers, WgslIdToMetadataMap,
        WgslInputData,
    },
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SingleBatchGpuCollisionDetectionSystemSet;

pub struct GpuCollisionSingleBatchRunnerPlugin;

impl Plugin for GpuCollisionSingleBatchRunnerPlugin {
    fn build(&self, app: &mut App) {
        let mut batched_collision_detection_schedule =
            Schedule::new(BatchedCollisionDetectionSchedule);
        batched_collision_detection_schedule.add_systems(
            (
                initialize_batch,
                update_wgsl_consts,
                update_pipeline,
                convert_collidables_to_wgsl_types,
                create_buffers,
                create_bind_group,
                dispatch_to_gpu,
                get_results_count_from_gpu,
                read_results_from_gpu,
                finish_batch,
            )
                .chain(),
        );
        app.add_schedule(batched_collision_detection_schedule)
            .add_systems(Startup, setup_single_batch_resources);
    }
}
