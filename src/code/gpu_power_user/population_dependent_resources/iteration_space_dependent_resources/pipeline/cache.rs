use bevy::{prelude::Resource, render::render_resource::ComputePipeline};

use crate::code::helpers::ecs::lru_cache::LruCache;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct PipelineKey {
    // pub max_num_outputs_hash: u64,
    // pub iteration_space_hash: u64,
    pub wgsl_hash: u64,
}

#[derive(Resource)]
pub struct PipelineCache {
    pub cache: LruCache<PipelineKey, ComputePipeline>,
}

impl PipelineCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(capacity),
        }
    }
}
