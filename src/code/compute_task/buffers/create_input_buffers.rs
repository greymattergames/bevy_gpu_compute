use std::any::{Any, TypeId};

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{DetectChanges, Entity, Event, EventReader, Query, Ref, Res, ResMut},
    render::{render_resource::Buffer, renderer::RenderDevice},
};
use bytemuck::Pod;
use wgpu::{BufferDescriptor, BufferUsages, util::BufferInitDescriptor};

use crate::code::compute_task::{
    component::TaskName,
    events::{GpuComputeTaskChangeEvent, InputDataChangeEvent},
    inputs::{
        input_data::{InputData, TypeErasedInputData},
        input_metadata_spec::InputVectorMetadataSpec,
        input_spec::{self, InputVectorTypesSpec},
    },
    wgsl_processable_types::{WgslCollisionResult, WgslCounter},
};

use super::components::InputBuffers;

pub fn create_input_buffers(
    mut tasks: Query<(
        &TaskName,
        &TypeErasedInputData,
        &InputVectorMetadataSpec,
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
        if let Ok((task_name, input_data, input_spec, mut buffers)) = task {
            buffers.0.clear();
            create_input_buffers_single_task(
                &task_name.get(),
                &render_device,
                &input_data,
                &input_spec,
                &mut buffers,
            );
        }
    }
}

fn create_input_buffers_single_task(
    task_name: &str,
    render_device: &Res<RenderDevice>,
    input_data: &TypeErasedInputData,
    input_spec: &InputVectorMetadataSpec,
    mut buffers: &mut InputBuffers,
) {
    // input buffers
    for (i, spec) in input_spec.get_all_metadata().iter().enumerate() {
        let label = format!("{}-input-{}", task_name, i);
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some(&label),
            contents: input_data.input_bytes(i).unwrap(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });
        buffers.0[i] = buffer;
        continue;
    }
}
