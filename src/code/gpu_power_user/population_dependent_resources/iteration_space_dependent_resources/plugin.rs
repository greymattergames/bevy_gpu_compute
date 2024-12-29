use bevy::{
    app::{App, Plugin, Startup},
    prelude::Commands,
};

use super::{
    pipeline::cache::PipelineCache,
    resources::{
        BatchCollidablePopulation, MaxNumGpuOutputItemsPerOutputType, NumGpuWorkgroupsRequired,
    },
};

pub struct GpuCollisionBatchSizeDependentResourcesPlugin;

impl Plugin for GpuCollisionBatchSizeDependentResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(MaxNumGpuOutputItemsPerOutputType::new(Default::default()));
    commands.insert_resource(NumGpuWorkgroupsRequired { x: 1, y: 1, z: 1 });
    commands.insert_resource(BatchCollidablePopulation(0));
    commands.insert_resource(PipelineCache::new(10));
}
