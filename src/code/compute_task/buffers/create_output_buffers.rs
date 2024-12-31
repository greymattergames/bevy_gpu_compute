use std::any::{Any, TypeId};

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{DetectChanges, Entity, Event, EventReader, Query, Ref, Res, ResMut, With},
    render::{render_resource::Buffer, renderer::RenderDevice},
};
use bytemuck::Pod;
use wgpu::{BufferDescriptor, BufferUsages, util::BufferInitDescriptor};

use crate::code::compute_task::{
    component::TaskName,
    events::{GpuComputeTaskChangeEvent, MaxOutputVectorLengthsChangedEvent},
    iteration_space_dependent_components::max_output_vector_lengths::MaxOutputVectorLengths,
    outputs::{
        output_data::OutputData,
        output_metadata_spec::{OutputVectorMetadata, OutputVectorMetadataSpec},
        output_spec::OutputVectorTypesSpec,
    },
    resources::GpuAcceleratedBevy,
    wgsl_processable_types::{WgslCollisionResult, WgslCounter},
};

use super::components::{InputBuffers, OutputBuffers, OutputCountBuffers, OutputStagingBuffers};

pub fn create_output_buffers(
    mut tasks: Query<
        (
            &TaskName,
            &OutputVectorMetadataSpec,
            Ref<MaxOutputVectorLengths>,
            &mut OutputBuffers,
            &mut OutputStagingBuffers,
            &mut OutputCountBuffers,
            &mut OutputStagingBuffers,
        ),
        With<GpuAcceleratedBevy>,
    >,
    mut output_limits_change_event_listener: EventReader<MaxOutputVectorLengthsChangedEvent>,
    render_device: &Res<RenderDevice>,
) {
    for (ev, _) in output_limits_change_event_listener
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        let task = tasks.get_mut(ev.entity().clone());
        if let Ok((
            task_name,
            output_spec,
            max_num_outputs,
            mut buffers,
            mut staging_buffers,
            mut count_buffers,
            mut count_staging_buffers,
        )) = task
        {
            buffers.0.clear();
            staging_buffers.0.clear();
            count_buffers.0.clear();
            count_staging_buffers.0.clear();
            create_output_buffers_single_task(
                task_name,
                render_device,
                output_spec,
                max_num_outputs,
                &mut buffers,
                &mut staging_buffers,
                &mut count_buffers,
                &mut count_staging_buffers,
            );
        }
    }
}

fn create_output_buffers_single_task(
    task_name: &TaskName,
    render_device: &Res<RenderDevice>,
    output_spec: &OutputVectorMetadataSpec,
    max_num_outputs: Ref<MaxOutputVectorLengths>,
    mut buffers: &mut OutputBuffers,
    mut staging_buffers: &mut OutputStagingBuffers,
    mut count_buffers: &mut OutputCountBuffers,
    mut count_staging_buffers: &mut OutputStagingBuffers,
) {
    for (i, output_spec) in output_spec.get_all_metadata().iter().enumerate() {
        if let Some(spec) = output_spec {
            create_output_buffer_single_output(
                render_device,
                task_name,
                i,
                spec,
                max_num_outputs.get(i),
                &mut buffers,
                &mut staging_buffers,
                &mut count_buffers,
                &mut count_staging_buffers,
            );
        }
    }
}

fn create_output_buffer_single_output(
    render_device: &Res<RenderDevice>,
    task_name: &TaskName,
    output_index: usize,
    output_spec: &OutputVectorMetadata,
    max_num_outputs: usize,
    mut buffers: &mut OutputBuffers,
    mut staging_buffers: &mut OutputStagingBuffers,
    mut count_buffers: &mut OutputCountBuffers,
    mut count_staging_buffers: &mut OutputStagingBuffers,
) {
    let output_size = output_spec.get_bytes() as u64 * max_num_outputs as u64;
    let output_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some(&format!("{:}-output-{:}", task_name.get(), output_index)),
        size: output_size,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    buffers.0.insert(output_index.clone(), output_buffer);
    let output_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!(
            "{:}-output-staging-{:}",
            task_name.get(),
            output_index
        )),
        size: output_size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    staging_buffers
        .0
        .insert(output_index.clone(), output_staging_buffer);
    if output_spec.get_include_count() {
        let counter = WgslCounter { count: 0 };
        let counter_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some(&format!(
                "{:}-output-counter-{:}",
                task_name.get(),
                output_index
            )),
            contents: bytemuck::cast_slice(&[counter]),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        });
        count_buffers.0.insert(output_index.clone(), counter_buffer);
        let counter_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!(
                "{:}-output-counter-staging-{:}",
                task_name.get(),
                output_index
            )),
            size: std::mem::size_of::<WgslCounter>() as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        count_staging_buffers
            .0
            .insert(output_index.clone(), counter_staging_buffer);
    }
}
