use bevy::{prelude::Resource, render::renderer::RenderDevice};
use wgpu::ShaderModule;

/**
 * Using this version the user must ensure the wgsl code contains the correct data input and output types and sizes.
 */

pub fn shader_module_from_wgsl_string(
    task_label: &str,
    wgsl_str: &str,
    render_device: &RenderDevice,
) -> ShaderModule {
    render_device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(task_label),
        source: wgpu::ShaderSource::Wgsl(wgsl_str.into()),
    })
}
