use std::collections::HashMap;

use bevy::{
    prelude::{Component, Resource},
    render::render_resource::BindGroup,
};
use wgpu::Buffer;

use crate::code::gpu_power_user::{
    entity_metadata::CollidableMetadata,
    wgsl_processable_types::{WgslDynamicPositions, WgslDynamicRadii},
};

use super::convert_collidables_to_wgsl_types::PerCollidableDataRequiredByGpu;

#[derive(Component)]
pub struct InputBuffers(pub HashMap<String, Buffer>);
#[derive(Component)]
pub struct OutputBuffers(pub HashMap<String, Buffer>);

#[derive(Component)]
pub struct OutputStagingBuffers(pub HashMap<String, Buffer>);
#[derive(Debug, Resource)]
pub struct GpuAccBevyBindGroup(pub Option<BindGroup>);

#[derive(Resource)]
pub struct WgslInputData {
    pub positions: WgslDynamicPositions,
    pub radii: WgslDynamicRadii,
}
impl Default for WgslInputData {
    fn default() -> Self {
        WgslInputData {
            positions: WgslDynamicPositions::default(),
            radii: WgslDynamicRadii::default(),
        }
    }
}

#[derive(Resource)]
pub struct CollidablesBatch(pub Vec<PerCollidableDataRequiredByGpu>);

#[derive(Component)]
pub struct OutputCountsFromGpu(pub HashMap<String, Option<usize>>);
#[derive(Resource)]
pub struct WgslIdToMetadataMap(pub Vec<CollidableMetadata>);

#[derive(Component)]

pub struct LatestResultsStore {
    pub results: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}
impl LatestResultsStore {
    pub fn get<T: 'static>(&self, label: &str) -> Option<&T> {
        self.results
            .get(label)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
}

// test
fn test() {
    let s = LatestResultsStore {
        results: HashMap::new(),
    };
    let v: &[u8] = s.get::<&[u8]>("test").unwrap();
}
