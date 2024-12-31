use std::hash::Hash;

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{Changed, DetectChanges, Entity, Event, EventReader, Query, Res, ResMut},
    render::renderer::RenderDevice,
};
use wgpu::ComputePipelineDescriptor;

use crate::code::compute_task::{
    component::TaskName,
    events::{GpuComputeTaskChangeEvent, WgslCodeChangedEvent},
    inputs::input_spec::InputVectorTypesSpec,
    outputs::output_spec::OutputVectorTypesSpec,
    pipeline_layout::PipelineLayout,
    wgsl_code::WgslCode,
};

use super::{
    cache::{PipelineCache, PipelineKey},
    shader_module::shader_module_from_wgsl_string,
};

pub fn update_pipeline(
    mut tasks: Query<
        (&TaskName, &WgslCode, &PipelineLayout, &mut PipelineCache),
        Changed<WgslCode>,
    >,
    mut wgsl_code_changed_event_reader: EventReader<WgslCodeChangedEvent>,
    render_device: Res<RenderDevice>,
) {
    for (ev, _) in wgsl_code_changed_event_reader
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        let task = tasks.get_mut(ev.entity().clone());
        if let Ok((task_name, wgsl, pipeline_layout, mut pipeline_cache)) = task {
            update_pipeline_for_wgsl_code(
                wgsl,
                task_name,
                &render_device,
                &pipeline_layout,
                &mut pipeline_cache,
            );
        }
    }
}

fn update_pipeline_for_wgsl_code(
    wgsl: &WgslCode,
    task_name: &TaskName,
    render_device: &Res<RenderDevice>,
    pipeline_layout: &PipelineLayout,
    pipeline_cache: &mut PipelineCache,
) {
    let key = PipelineKey {
        wgsl_hash: wgsl.code_hash as u64,
    };
    if pipeline_cache.cache.contains_key(&key) {
        return;
    } else {
        let shader_module =
            shader_module_from_wgsl_string(&task_name.get(), &wgsl.code(), &render_device);
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
