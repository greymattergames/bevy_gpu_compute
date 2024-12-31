use std::any::{Any, TypeId};

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{DetectChanges, Query, Ref, Res, ResMut},
    render::{render_resource::Buffer, renderer::RenderDevice},
};
use bytemuck::Pod;
use wgpu::{BufferDescriptor, BufferUsages, util::BufferInitDescriptor};

use crate::code::compute_task::{
    component::{GpuComputeTask, TaskName},
    inputs::{input_data::InputData, input_spec::InputVectorTypesSpec},
    wgsl_processable_types::{WgslCollisionResult, WgslCounter},
};

use super::components::InputBuffers;

pub fn create_input_buffers<I: InputVectorTypesSpec + 'static + Send + Sync>(
    mut tasks: Query<(&TaskName, Ref<InputData<I>>, &mut InputBuffers)>,
    render_device: Res<RenderDevice>,
) {
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(|(task_name, input_data, mut buffers)| {
            //todo, change these to work off of events
            if input_data.is_changed() || input_data.is_changed() {
                buffers.0.clear();
                create_input_buffers_single_task(
                    &task_name.0,
                    &render_device,
                    &input_data,
                    &mut buffers,
                );
            }
        });
}

fn create_input_buffers_single_task<I: InputVectorTypesSpec + 'static + Send + Sync>(
    task_name: &str,
    render_device: &Res<RenderDevice>,
    input_data: &InputData<I>,
    mut buffers: &mut InputBuffers,
) {
    // input buffers
    for (i, spec) in InputData::<I>::get_all_metadata().iter().enumerate() {
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
