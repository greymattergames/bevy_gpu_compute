use bevy::prelude::*;
use bevy::render::render_resource::BufferUsages;
use bevy::render::renderer::RenderDevice;

use super::get_collidables::get_collidables;
use super::resources::{
    AllCollidablesThisFrame, BindGroupLayoutsResource, CounterStagingBuffer, MaxBatchSize,
    MaxDetectableCollisionsScale, PipelineLayoutResource,
};
use super::single_batch::plugin::GpuCollisionSingleBatchRunnerPlugin;
use super::wgsl_processable_types::{WgslCollisionResult, WgslCounter};

pub struct GpuAccelerationBevyPowerUserPlugin {}

impl GpuAccelerationBevyPowerUserPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for GpuAccelerationBevyPowerUserPlugin {
    fn build(&self, app: &mut App) {
        let max_detectable_collisions_scale = self.max_detectable_collisions_scale;
        let workgroup_size = self.workgroup_size;
        app.add_plugins(GpuCollisionPopDependentResourcesPlugin)
            .add_plugins(GpuCollisionSingleBatchRunnerPlugin)
            .add_systems(
                Startup,
                (
                    // create max_detectable_collisions_scale resource
                    move |mut commands: Commands| {
                        commands.insert_resource(MaxDetectableCollisionsScale(
                            max_detectable_collisions_scale,
                        ));
                        commands.insert_resource(WorkgroupSize(workgroup_size));
                        commands.insert_resource(MaxBatchSize(10));
                        commands.insert_resource(AllCollidablesThisFrame(Vec::new()));
                    },
                    setup_multi_batch_manager_resources,
                    create_persistent_gpu_resources,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    update_max_batch_size,
                    get_collidables,
                    generate_batch_jobs,
                    run_batched_collision_detection_schedule,
                    combine_results,
                )
                    .chain()
                    .before(process_collisions),
            );
    }
}
