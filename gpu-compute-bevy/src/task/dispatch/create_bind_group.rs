use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{Component, Query, Res},
    render::{render_resource::BindGroup, renderer::RenderDevice},
};

use crate::task::{
    buffers::components::{InputBuffers, OutputBuffers, OutputCountBuffers},
    inputs::input_vector_metadata_spec::InputVectorsMetadataSpec,
    outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
    task_components::{bind_group_layouts::BindGroupLayouts, task_name::TaskName},
    task_specification::task_specification::ComputeTaskSpecification,
};

/**
Binding the buffers to the corresponding wgsl code.

For example, this might be the wgsl code:
```wgsl

@group(0) @binding(0) var<storage, read> positions: Positions;
@group(0) @binding(1) var<storage, read> radii: Radii;
@group(0) @binding(2) var<storage, read_write> results: CollisionResults;
```

The numbers in the `@binding` are the bind group entry numbers. The `@group` is the bind group number. We are only using a single bind group in the current library version.
 */

#[derive(Default, Component)]
pub struct BindGroupComponent(pub Option<BindGroup>);

pub fn create_bind_groups(
    mut tasks: Query<(
        &TaskName,
        &ComputeTaskSpecification,
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
                task_spec,
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
                    task_spec.input_vectors_metadata_spec(),
                    task_spec.output_vectors_metadata_spec(),
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
    render_device: &RenderDevice,
    bind_group_layouts: &BindGroupLayouts,  // when this changes
    input_specs: &InputVectorsMetadataSpec, // when binding number changes
    output_specs: &OutputVectorsMetadataSpec, // when binding number changes, or include count or count binding number
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
