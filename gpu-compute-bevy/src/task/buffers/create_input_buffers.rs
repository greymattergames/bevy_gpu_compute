use bevy::{
    ecs::batching::BatchingStrategy,
    log::{self, info},
    prelude::{EventReader, Query, Res},
    render::renderer::RenderDevice,
};
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::{
    events::{GpuComputeTaskChangeEvent, InputDataChangeEvent},
    inputs::{
        input_vector_metadata_spec::InputVectorsMetadataSpec,
        type_erased_input_data::TypeErasedInputData,
    },
    task_components::task_name::TaskName,
    task_specification::task_specification::ComputeTaskSpecification,
};

use super::components::InputBuffers;

pub fn create_input_buffers(
    mut tasks: Query<(
        &TaskName,
        &TypeErasedInputData,
        &ComputeTaskSpecification,
        &mut InputBuffers,
    )>,
    mut input_data_change_event_listener: EventReader<InputDataChangeEvent>,
    render_device: Res<RenderDevice>,
) {
    for (ev, _) in input_data_change_event_listener
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        let task = tasks.get_mut(ev.entity().clone());
        if let Ok((task_name, input_data, task_spec, mut buffers)) = task {
            buffers.0.clear();
            create_input_buffers_single_task(
                &task_name.get(),
                &render_device,
                &input_data,
                &task_spec.input_vectors_metadata_spec(),
                &mut buffers,
            );
        }
    }
}

fn create_input_buffers_single_task(
    task_name: &str,
    render_device: &RenderDevice,
    input_data: &TypeErasedInputData,
    input_spec: &InputVectorsMetadataSpec,
    buffers: &mut InputBuffers,
) {
    buffers.0.clear();
    for (i, spec) in input_spec.get_all_metadata().iter().enumerate() {
        if let Some(s) = spec {
            let label = format!("{}-input-{}", task_name, s.name().name());
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(&label),
                contents: input_data.input_bytes(i).unwrap(),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });
            info!(
                "Created input buffer for task {} with label {}",
                task_name, label
            );
            buffers.0.push(buffer);
            continue;
        }
    }
}
