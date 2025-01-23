use bevy::{log, render::renderer::RenderDevice};

use crate::task::task_components::task::BevyGpuComputeTask;

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

pub fn create_bind_group(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    log::info!("Creating bind group for task {}", task.name());
    let mut bindings = Vec::new();
    for (i, spec) in task
        .spec
        .config_input_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
    {
        if let Some(s) = spec {
            if let Some(conf_in_buff) = task.buffers.config_input.get(i) {
                bindings.push(wgpu::BindGroupEntry {
                    binding: s.get_binding_number(),
                    resource: conf_in_buff.as_entire_binding(),
                });
            } else {
                panic!("Config input has not been set for task {}", task.name());
            }
        }
    }
    for (i, spec) in task
        .spec
        .input_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
    {
        if let Some(s) = spec {
            if let Some(buffer) = task.buffers.input.get(i) {
                bindings.push(wgpu::BindGroupEntry {
                    binding: s.get_binding_number(),
                    resource: buffer.as_entire_binding(),
                });
            } else {
                panic!(
                    "Input has not been set for task {}, with index: {}. Input buffers: {:?}",
                    task.name(),
                    i,
                    task.buffers.input
                );
            }
        }
    }
    for (i, spec) in task
        .spec
        .output_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
    {
        if let Some(s) = spec {
            let output_buffer = task.buffers.output.get(i).unwrap();
            bindings.push(wgpu::BindGroupEntry {
                binding: s.get_binding_number(),
                resource: output_buffer.as_entire_binding(),
            });
            if s.get_include_count() {
                let count_buffer = task.buffers.output_count.get(i).unwrap();
                bindings.push(wgpu::BindGroupEntry {
                    binding: s.get_count_binding_number().unwrap(),
                    resource: count_buffer.as_entire_binding(),
                });
            }
        }
    }
    task.bind_group = Some(render_device.create_bind_group(
        task.name(),
        &task.bind_group_layout.as_ref().unwrap(),
        &bindings,
    ));
}
