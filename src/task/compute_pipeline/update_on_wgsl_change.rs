use bevy::{
    ecs::batching::BatchingStrategy,
    log,
    prelude::{Changed, EventReader, Query, Res},
    render::renderer::RenderDevice,
};
use wgpu::ComputePipelineDescriptor;

use crate::task::{
    events::{GpuComputeTaskChangeEvent, WgslCodeChangedEvent},
    task_components::task_name::TaskName,
    task_specification::task_specification::TaskUserSpecification,
    wgsl_code::WgslCode,
};

use super::{
    cache::{PipelineKey, PipelineLruCache},
    pipeline_layout::PipelineLayoutComponent,
    shader_module::shader_module_from_wgsl_string,
};

pub fn update_pipelines_on_wgsl_change(
    mut tasks: Query<(
        &TaskName,
        &TaskUserSpecification,
        &PipelineLayoutComponent,
        &mut PipelineLruCache,
    )>,
    mut wgsl_code_changed_event_reader: EventReader<WgslCodeChangedEvent>,
    render_device: Res<RenderDevice>,
) {
    for (ev, _) in wgsl_code_changed_event_reader
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        let task = tasks.get_mut(ev.entity().clone());
        if let Ok((task_name, task_spec, pipeline_layout, mut pipeline_cache)) = task {
            update_single_pipeline(
                task_spec.wgsl_code(),
                task_name,
                &render_device,
                &pipeline_layout,
                &mut pipeline_cache,
            );
        }
    }
}

fn update_single_pipeline(
    wgsl: &WgslCode,
    task_name: &TaskName,
    render_device: &RenderDevice,
    pipeline_layout: &PipelineLayoutComponent,
    pipeline_cache: &mut PipelineLruCache,
) {
    log::info!("Updating pipeline for task {}", task_name.get());
    let key = PipelineKey {
        wgsl_hash: wgsl.code_hash as u64,
    };
    if pipeline_cache.cache.contains_key(&key) {
        return;
    } else {
        log::info!("Creating new pipeline for task {}", task_name.get());
        let shader_module =
            shader_module_from_wgsl_string(&task_name.get(), &wgsl.code(), &render_device);
        log::info!("Shader module created");
        log::info!(" layout {:?}", pipeline_layout.0);
        let compute_pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&task_name.get()),
            layout: Some(&pipeline_layout.0),
            module: &shader_module,
            entry_point: Some(wgsl.entry_point_function_name()),
            compilation_options: Default::default(),
            cache: None,
        });
        pipeline_cache.cache.insert(key, compute_pipeline);
    }
}
