use std::any::{Any, TypeId};

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{DetectChanges, Query, Ref, Res, ResMut},
    render::{render_resource::Buffer, renderer::RenderDevice},
};
use bytemuck::Pod;
use wgpu::{BufferDescriptor, BufferUsages, util::BufferInitDescriptor};

use crate::code::gpu_power_user::{
    inputs::{input_data::InputData, input_specs::InputSpecs},
    wgsl_processable_types::{WgslCollisionResult, WgslCounter},
};

use super::misc_components::InputBuffers;

pub fn create_input_buffers(
    mut tasks: Query<(Ref<InputSpecs>, Ref<InputData>, &mut InputBuffers)>,
    render_device: Res<RenderDevice>,
) {
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(|(input_specs, input_data, mut buffers)| {
            if input_specs.is_changed() || input_data.is_changed() {
                buffers.0.clear();
                create_input_buffers_single_task(
                    &render_device,
                    input_specs,
                    input_data,
                    &mut buffers,
                );
            }
        });
}

fn create_input_buffers_single_task(
    render_device: &Res<RenderDevice>,
    input_specs: Ref<InputSpecs>,
    input_data: Ref<InputData>,
    mut buffers: &mut InputBuffers,
) {
    // input buffers
    for (label, spec) in input_specs.specs.iter() {
        if !spec.skip_validation {
            if let Err(err) = input_specs.validate_data(&input_data) {
                panic!("Input validation failed: {}", err);
            }
        }
        let data = input_data
            .data
            .get(label)
            .expect("Missing input data for label");

        // We need to match the type_id to create the correct buffer
        // This can be done with a macro to avoid repetition
        macro_rules! handle_type {
            ($($t:ty),*) => {
                $(
                    if spec.type_id == TypeId::of::<Vec<$t>>() {
                        if let Some(buffer) = create_input_buffer_for_type::<$t>(
                            &render_device,
                            label,
                            data,
                        ) {
                            buffers.0.insert(label.clone(), buffer);
                            continue;
                        }
                    }
                )*
            }
        }

        // Register all your possible types here
        handle_type!(
            f32, // example types
            u32,
            i32,
            WgslCollisionResult // your custom types
                                // Add more types as needed
        );
    }
}
fn create_input_buffer_for_type<T: Pod>(
    render_device: &RenderDevice,
    label: &str,
    data: &Box<dyn Any + Send + Sync>,
) -> Option<Buffer> {
    data.downcast_ref::<Vec<T>>().map(|typed_data| {
        render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(typed_data),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        })
    })
}
