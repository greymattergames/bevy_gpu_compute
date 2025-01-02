use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{EventReader, Query, Ref, Res},
    render::renderer::RenderDevice,
};
use wgpu::{BufferDescriptor, BufferUsages, util::BufferInitDescriptor};

use crate::task::{
    events::{GpuComputeTaskChangeEvent, IterationSpaceOrMaxOutVecLengthChangedEvent},
    outputs::definitions::{
        max_output_vector_lengths::MaxOutputVectorLengths,
        output_vector_metadata_spec::{OutputVectorMetadata, OutputVectorsMetadataSpec},
        wgsl_counter::WgslCounter,
    },
    task_components::task_name::TaskName,
    task_specification::task_specification::TaskUserSpecification,
};

use super::components::{
    OutputBuffers, OutputCountBuffers, OutputCountStagingBuffers, OutputStagingBuffers,
};

pub fn create_output_buffers(
    mut tasks: Query<(
        &TaskName,
        Ref<TaskUserSpecification>,
        &mut OutputBuffers,
        &mut OutputStagingBuffers,
        &mut OutputCountBuffers,
        &mut OutputCountStagingBuffers,
    )>,
    mut output_limits_change_event_listener: EventReader<
        IterationSpaceOrMaxOutVecLengthChangedEvent,
    >,
    render_device: Res<RenderDevice>,
) {
    for (ev, _) in output_limits_change_event_listener
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        let task = tasks.get_mut(ev.entity().clone());
        if let Ok((
            task_name,
            task_spec,
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
                &render_device,
                task_spec.output_vectors_metadata_spec(),
                task_spec.max_output_vector_lengths(),
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
    render_device: &RenderDevice,
    output_spec: &OutputVectorsMetadataSpec,
    max_output_vector_lengths: &MaxOutputVectorLengths,
    mut buffers: &mut OutputBuffers,
    mut staging_buffers: &mut OutputStagingBuffers,
    mut count_buffers: &mut OutputCountBuffers,
    mut count_staging_buffers: &mut OutputCountStagingBuffers,
) {
    for (i, output_spec) in output_spec.get_all_metadata().iter().enumerate() {
        if let Some(spec) = output_spec {
            create_output_buffer_single_output(
                render_device,
                task_name,
                i,
                spec,
                max_output_vector_lengths.get(i),
                &mut buffers,
                &mut staging_buffers,
                &mut count_buffers,
                &mut count_staging_buffers,
            );
        }
    }
}

fn create_output_buffer_single_output(
    render_device: &RenderDevice,
    task_name: &TaskName,
    output_index: usize,
    output_spec: &OutputVectorMetadata,
    max_output_vector_lengths: usize,
    buffers: &mut OutputBuffers,
    staging_buffers: &mut OutputStagingBuffers,
    count_buffers: &mut OutputCountBuffers,
    count_staging_buffers: &mut OutputCountStagingBuffers,
) {
    let output_size = output_spec.get_bytes() as u64 * max_output_vector_lengths as u64;
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
