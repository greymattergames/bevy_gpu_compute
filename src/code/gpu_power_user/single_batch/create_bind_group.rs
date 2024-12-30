use bevy::{
    core_pipeline::core_2d::graph::input,
    prelude::{Res, ResMut},
    render::renderer::RenderDevice,
};

use crate::code::gpu_power_user::resources::BindGroupLayoutsResource;

use super::resources::{BindGroup, SingleBatchBuffers};

/**
 * Binding the buffers to the corresponding wgsl code
 */
pub fn create_bind_group(
    render_device: Res<RenderDevice>,
    bind_group_layouts: Res<BindGroupLayoutsResource>,
    input_buffers: Res<BevyGpuAccelerationInputBuffers>,
    output_buffers: Res<BevyGpuAccelerationOutputBuffers>,
    mut bind_group_res: ResMut<BindGroup>,
) {
    let bindings = Vec::new();
    let mut count: u32 = 0;
    for input_buffer in input_buffers.iter() {
        bindings.push(wgpu::BindGroupEntry {
            binding: if input_buffer.binding_number.is_some() {
                count = input_buffer.binding_number.unwrap();
                input_buffer.binding_number.unwrap()
            } else {
                count
            },
            resource: input_buffer.buffer.as_ref().unwrap().as_entire_binding(),
        });
        count += 1;
    }
    for output_buffer in output_buffers.iter() {
        bindings.push(wgpu::BindGroupEntry {
            binding: if output_buffer.binding_number.is_some() {
                count = output_buffer.binding_number.unwrap();
                output_buffer.binding_number.unwrap()
            } else {
                count
            },
            resource: output_buffer.buffer.as_ref().unwrap().as_entire_binding(),
        });
        count += 1;
    }
    bind_group_res.0 = Some(render_device.create_bind_group(
        Some("Bevy GPU Acceleration Bind Group"),
        &bind_group_layouts.0,
        &bindings,
    ));
}
