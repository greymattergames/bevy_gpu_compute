use bevy::{
    app::{App, Plugin, Startup},
    prelude::Commands,
};

use crate::code::gpu_easy::population_dependent_resources::batch_size_dependent_resources::resources::BatchCollidablePopulation;

use super::{
    iteration_space::IterationSpace,
    max_num_outputs_per_type::MaxNumGpuOutputItemsPerOutputType,
    pipeline::cache::PipelineCache,
    workgroup_sizes::{NumGpuWorkgroupsRequired, WorkgroupSizes},
};

pub struct GpuAccBevyIterSpaceDependentResourcesPlugin;

impl Plugin for GpuAccBevyIterSpaceDependentResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GpuAccBevyIterSpaceDependentResourcesPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(IterationSpace { x: 1, y: 1, z: 1 });
    commands.insert_resource(WorkgroupSizes::one_d());
    commands.insert_resource(MaxNumGpuOutputItemsPerOutputType::new(Default::default()));
    commands.insert_resource(NumGpuWorkgroupsRequired { x: 1, y: 1, z: 1 });
    commands.insert_resource(PipelineCache::new(10));
}
