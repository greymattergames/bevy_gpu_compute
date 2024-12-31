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
    inputs::input_specs::InputSpecs,
    outputs::output_spec::OutputSpecs,
    resources::TaskLabel,
};

/**
 * Binding the buffers to the corresponding wgsl code
 */

#[derive(Default, Component)]
pub struct BindGroupComponent(pub Option<BindGroup>);

pub fn create_bind_groups(
    mut tasks: Query<(
        &TaskLabel,
        &OutputSpecs,
        &InputSpecs,
        &BindGroupLayouts,
        &InputBuffers,
        &OutputCountBuffers,
        &OutputBuffers,
        &mut BindGroupComponent,
    )>,
    render_device: Res<RenderDevice>,
) {
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                task_label,
                output_specs,
                input_specs,
                bind_group_layouts,
                input_buffers,
                output_count_buffers,
                output_buffers,
                mut bind_group_res,
            )| {
                create_bind_group_single_task(
                    task_label,
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
    task_label: &TaskLabel, //when this changes
    render_device: &Res<RenderDevice>,
    bind_group_layouts: &BindGroupLayouts, // when this changes
    input_specs: &InputSpecs,              // when binding number changes
    output_specs: &OutputSpecs, // when binding number changes, or include count or count binding number
    input_buffers: &InputBuffers,
    output_count_buffers: &OutputCountBuffers,
    output_buffers: &OutputBuffers,
    mut bind_group_component: &mut BindGroupComponent,
) {
    // todo only run when necessary
    let mut bindings = Vec::new();
    for (label, spec) in input_specs.specs.iter() {
        let buffer = input_buffers.0.get(label).unwrap();
        bindings.push(wgpu::BindGroupEntry {
            binding: spec.binding_number,
            resource: buffer.as_entire_binding(),
        });
    }
    for (label, spec) in output_specs.specs.iter() {
        let output_buffer = output_buffers.0.get(label).unwrap();
        bindings.push(wgpu::BindGroupEntry {
            binding: spec.binding_number,
            resource: output_buffer.as_entire_binding(),
        });
        if spec.include_count {
            let count_buffer = output_count_buffers.0.get(label).unwrap();
            bindings.push(wgpu::BindGroupEntry {
                binding: spec.count_binding_number.unwrap(),
                resource: count_buffer.as_entire_binding(),
            });
        }
    }
    bind_group_component.0 = Some(render_device.create_bind_group(
        task_label.0.as_str(),
        &bind_group_layouts.0,
        &bindings,
    ));
}
