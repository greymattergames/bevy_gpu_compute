use bevy::{
    ecs::batching::BatchingStrategy,
    log,
    prelude::{EventReader, Query, Res},
    render::renderer::RenderDevice,
};

use bevy_gpu_compute_core::TypesSpec;
use wgpu::{ComputePipelineDescriptor, PipelineCompilationOptions};

use crate::task::{task_commands::GpuTaskCommands, task_components::task::BevyGpuComputeTask};

use super::cache::{PipelineKey, PipelineLruCache};

pub fn update_compute_pipeline(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    if task.input_array_lengths.is_none() {
        return;
    }
    log::info!("Updating pipeline for task {}", task.name());
    let key = PipelineKey {
        pipeline_consts_version: task.spec.iter_space_and_out_lengths_version(),
    };
    if task.pipeline_cache.cache.contains_key(&key) {
        return;
    } else {
        log::info!("Creating new pipeline for task {}", task.name());
        log::info!(" layout {:?}", task.pipeline_layout);
        let compute_pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&task.name()),
            layout: task.pipeline_layout.as_ref(),
            module: task.spec.wgsl_code().shader_module(),
            entry_point: Some(task.spec.wgsl_code().entry_point_function_name()),
            // this is where we specify new values for pipeline constants...
            compilation_options: PipelineCompilationOptions {
                constants: &&task
                    .spec
                    .get_pipeline_consts(task.input_array_lengths.as_ref().unwrap()),
                zero_initialize_workgroup_memory: Default::default(),
            },
            cache: None,
        });
        task.pipeline_cache.cache.insert(key, compute_pipeline);
    }
}
