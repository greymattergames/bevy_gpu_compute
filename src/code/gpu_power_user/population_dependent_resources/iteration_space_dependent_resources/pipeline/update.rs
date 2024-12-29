use std::hash::Hash;

use bevy::{
    prelude::{DetectChanges, Res, ResMut},
    render::renderer::RenderDevice,
};
use wgpu::ComputePipelineDescriptor;

use crate::code::gpu_power_user::{
    population_dependent_resources::{
        iteration_space_dependent_resources::resources::MaxNumGpuOutputItemsPerOutputType,
        resources::{IterationSpace, WorkgroupSizes},
    },
    resources::{PipelineLayoutResource, WgslCode},
};

use super::{
    cache::{PipelineCache, PipelineKey},
    shader_module::shader_module_from_wgsl_string,
};

pub fn update_pipeline(
    wgsl: Res<WgslCode>,
    render_device: Res<RenderDevice>,
    pipeline_layout_resource: Res<PipelineLayoutResource>,
    mut pipeline_cache: ResMut<PipelineCache>,
) {
    if wgsl.is_changed() {
        let key = PipelineKey {
            wgsl_hash: wgsl.code_hash as u64,
        };
        if pipeline_cache.cache.contains_key(&key) {
            return;
        } else {
            let shader_module = shader_module_from_wgsl_string(&wgsl.code(), &render_device);
            let compute_pipeline =
                render_device.create_compute_pipeline(&ComputePipelineDescriptor {
                    label: Some("Gpu Acceleration Bevy Pipeline"), //todo, name this after the specific compute task
                    layout: Some(&pipeline_layout_resource.0), // Use the pipeline layout instead of None
                    module: &shader_module,
                    entry_point: Some(wgsl.entry_point_function_name()),
                    compilation_options: Default::default(),
                    cache: None,
                });
            pipeline_cache.cache.insert(key, compute_pipeline);
        }
    }
}
