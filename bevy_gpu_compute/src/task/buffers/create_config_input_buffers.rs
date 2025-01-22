use bevy::{
    ecs::batching::BatchingStrategy,
    log::info,
    prelude::{EventReader, Query, Res},
    render::renderer::RenderDevice,
};
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::{
    events::{ConfigInputDataChangeEvent, InputDataChangeEvent},
    inputs::config_type::{
        config_input_metadata_spec::ConfigInputsMetadataSpec,
        type_erased_config_input_data::TypeErasedConfigInputData,
    },
    task_components::task_name::TaskName,
    task_specification::task_specification::ComputeTaskSpecification,
};

use super::components::{ConfigInputBuffers, InputBuffers};

pub fn create_config_input_buffers(
    mut tasks: Query<(
        &TaskName,
        &TypeErasedConfigInputData,
        &ComputeTaskSpecification,
        &mut ConfigInputBuffers,
    )>,
    mut input_config_data_change_event_listener: EventReader<ConfigInputDataChangeEvent>,
    render_device: Res<RenderDevice>,
) {
    for (ev, _) in input_config_data_change_event_listener
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
                &task_spec.config_input_metadata_spec(),
                &mut buffers,
            );
        }
    }
}

fn create_input_buffers_single_task(
    task_name: &str,
    render_device: &RenderDevice,
    input_data: &TypeErasedConfigInputData,
    input_spec: &ConfigInputsMetadataSpec,
    buffers: &mut ConfigInputBuffers,
) {
    buffers.0.clear();
    for (i, spec) in input_spec.get_all_metadata().iter().enumerate() {
        if let Some(s) = spec {
            let label = format!("{}-input-{}", task_name, s.name().name());
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(&label),
                contents: input_data.input_bytes(i).unwrap(),
                usage: BufferUsages::UNIFORM,
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
