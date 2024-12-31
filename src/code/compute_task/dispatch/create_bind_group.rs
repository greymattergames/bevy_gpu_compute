use core::task;

use bevy::{
    core_pipeline::core_2d::graph::input,
    ecs::batching::BatchingStrategy,
    prelude::{Component, Query, Res, ResMut},
    render::{render_resource::BindGroup, renderer::RenderDevice},
};

use crate::code::compute_task::{
    bind_group_layouts::BindGroupLayouts,
    buffers::components::{InputBuffers, OutputBuffers, OutputCountBuffers},
    component::TaskName,
    inputs::input_metadata_spec::InputVectorMetadataSpec,
    outputs::output_metadata_spec::{OutputVectorMetadata, OutputVectorMetadataSpec},
};

/**
 * Binding the buffers to the corresponding wgsl code
 */

#[derive(Default, Component)]
pub struct BindGroupComponent(pub Option<BindGroup>);

pub fn create_bind_groups(
    mut tasks: Query<(
        &TaskName,
        &OutputVectorMetadataSpec,
        &InputVectorMetadataSpec,
        &BindGroupLayouts,
        &InputBuffers,
        &OutputCountBuffers,
        &OutputBuffers,
        &mut BindGroupComponent,
    )>,
    render_device: Res<RenderDevice>,
) {
    // must run for every run of each task
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                task_name,
                output_specs,
                input_specs,
                bind_group_layouts,
                input_buffers,
                output_count_buffers,
                output_buffers,
                mut bind_group_res,
            )| {
                create_bind_group_single_task(
                    task_name,
                    &render_device,
                    bind_group_layouts,
                    input_specs,
                    output_specs,
                    input_buffers,
                    output_count_buffers,
                    output_buffers,
                    &mut bind_group_res,
                );
            },
        )
}

fn create_bind_group_single_task(
    task_name: &TaskName, //when this changes
    render_device: &Res<RenderDevice>,
    bind_group_layouts: &BindGroupLayouts, // when this changes
    input_specs: &InputVectorMetadataSpec, // when binding number changes
    output_specs: &OutputVectorMetadataSpec, // when binding number changes, or include count or count binding number
    input_buffers: &InputBuffers,
    output_count_buffers: &OutputCountBuffers,
    output_buffers: &OutputBuffers,
    mut bind_group_component: &mut BindGroupComponent,
) {
    let mut bindings = Vec::new();
    for (i, spec) in input_specs.get_all_metadata().iter().enumerate() {
        if let Some(s) = spec {
            let buffer = input_buffers.0.get(i).unwrap();
            bindings.push(wgpu::BindGroupEntry {
                binding: s.get_binding_number(),
                resource: buffer.as_entire_binding(),
            });
        }
    }
    for (i, spec) in output_specs.get_all_metadata().iter().enumerate() {
        if let Some(s) = spec {
            let output_buffer = output_buffers.0.get(i).unwrap();
            bindings.push(wgpu::BindGroupEntry {
                binding: s.get_binding_number(),
                resource: output_buffer.as_entire_binding(),
            });
            if s.get_include_count() {
                let count_buffer = output_count_buffers.0.get(i).unwrap();
                bindings.push(wgpu::BindGroupEntry {
                    binding: s.get_count_binding_number().unwrap(),
                    resource: count_buffer.as_entire_binding(),
                });
            }
        }
    }
    bind_group_component.0 =
        Some(render_device.create_bind_group(task_name.get(), &bind_group_layouts.0, &bindings));
}
