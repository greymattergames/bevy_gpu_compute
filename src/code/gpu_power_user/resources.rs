use std::hash::{Hash, Hasher};

use bevy::{
    prelude::{Component, Resource},
    render::render_resource::{BindGroupLayout, Buffer},
};

use super::single_batch::convert_collidables_to_wgsl_types::PerCollidableDataRequiredByGpu;

#[derive(Resource)]
pub struct WgslCode {
    code: String,
    entry_point_function_name: String,
    pub code_hash: u64,
}

impl WgslCode {
    pub fn new(wgsl_code: String, entry_point_function_name: String) -> Self {
        Self {
            code: wgsl_code.clone(),
            entry_point_function_name,
            code_hash: {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                wgsl_code.hash(&mut hasher);
                hasher.finish()
            },
        }
    }
    pub fn from_file(file_path: &str, entry_point_function_name: String) -> Self {
        let code = std::fs::read_to_string(file_path).unwrap();
        Self::new(code, entry_point_function_name)
    }
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn entry_point_function_name(&self) -> &str {
        &self.entry_point_function_name
    }
}
// Resources to store reusable GPU state

#[derive(Resource)]
pub struct BindGroupLayoutsResource(pub BindGroupLayout);

#[derive(Resource)]
pub struct PipelineLayoutResource(pub wgpu::PipelineLayout);

#[derive(Resource)]
pub struct CounterStagingBuffer(pub Buffer);

#[derive(Debug, Resource)]
pub struct MaxDetectableCollisionsScale(pub f32);

#[derive(Clone, Resource)]
pub struct AllCollidablesThisFrame(pub Vec<PerCollidableDataRequiredByGpu>);

#[derive(Resource)]
pub struct MaxBatchSize(pub usize);

#[derive(Component)]
///Flag component
pub struct GpuAccBevy {}
