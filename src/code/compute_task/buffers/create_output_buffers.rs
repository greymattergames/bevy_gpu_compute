use std::any::{Any, TypeId};

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{DetectChanges, Query, Ref, Res, ResMut, With},
    render::{render_resource::Buffer, renderer::RenderDevice},
};
use bytemuck::Pod;
use wgpu::{BufferDescriptor, BufferUsages, util::BufferInitDescriptor};

use crate::code::compute_task::{
    inputs::{input_data::InputData, input_specs::InputSpecs},
    iteration_space_dependent_resources::max_num_outputs_per_type::MaxNumGpuOutputItemsPerOutputType,
    outputs::output_spec::{OutputSpec, OutputSpecs},
    resources::GpuAcceleratedBevy,
    wgsl_processable_types::{WgslCollisionResult, WgslCounter},
};

use super::components::{InputBuffers, OutputBuffers, OutputCountBuffers, OutputStagingBuffers};

pub fn create_output_buffers(
    mut tasks: Query<
        (
            Ref<OutputSpecs>,
            Ref<MaxNumGpuOutputItemsPerOutputType>,
            &mut OutputBuffers,
            &mut OutputStagingBuffers,
            &mut OutputCountBuffers,
            &mut OutputStagingBuffers,
        ),
        With<GpuAcceleratedBevy>,
    >,
    render_device: &Res<RenderDevice>,
) {
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                output_specs,
                max_num_outputs,
                mut buffers,
                mut staging_buffers,
                mut count_buffers,
                mut count_staging_buffers,
            )| {
                if output_specs.is_changed() || max_num_outputs.is_changed() {
                    buffers.0.clear();
                    staging_buffers.0.clear();
                    count_buffers.0.clear();
                    count_staging_buffers.0.clear();
                    create_output_buffers_single_task(
                        render_device,
                        output_specs,
                        max_num_outputs,
                        &mut buffers,
                        &mut staging_buffers,
                        &mut count_buffers,
                        &mut count_staging_buffers,
                    );
                }
            },
        );
}

fn create_output_buffers_single_task(
    render_device: &Res<RenderDevice>,
    output_specs: Ref<OutputSpecs>,
    max_num_outputs: Ref<MaxNumGpuOutputItemsPerOutputType>,
    mut buffers: &mut OutputBuffers,
    mut staging_buffers: &mut OutputStagingBuffers,
    mut count_buffers: &mut OutputCountBuffers,
    mut count_staging_buffers: &mut OutputStagingBuffers,
) {
    for (label, output_spec) in output_specs.specs.iter() {
        create_output_buffer_single_output(
            render_device,
            label,
            output_spec,
            max_num_outputs.get(&label),
            &mut buffers,
            &mut staging_buffers,
            &mut count_buffers,
            &mut count_staging_buffers,
        );
    }
}

fn create_output_buffer_single_output(
    render_device: &Res<RenderDevice>,
    output_label: &String,
    output_spec: &OutputSpec,
    max_num_outputs: usize,
    mut buffers: &mut OutputBuffers,
    mut staging_buffers: &mut OutputStagingBuffers,
    mut count_buffers: &mut OutputCountBuffers,
    mut count_staging_buffers: &mut OutputStagingBuffers,
) {
    let results_size = output_spec.item_bytes as u64 * max_num_outputs as u64;
    let results_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some(&output_label),
        size: results_size,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    buffers.0.insert(output_label.clone(), results_buffer);
    let results_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("{:} Staging", output_label)),
        size: results_size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    staging_buffers
        .0
        .insert(output_label.clone(), results_staging_buffer);
    if output_spec.include_count {
        let counter = WgslCounter { count: 0 };
        let counter_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some(&format!("{:} Counter", output_label)),
            contents: bytemuck::cast_slice(&[counter]),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        });
        count_buffers.0.insert(output_label.clone(), counter_buffer);
        let counter_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{:} Counter Staging", output_label)),
            size: std::mem::size_of::<WgslCounter>() as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        count_staging_buffers
            .0
            .insert(output_label.clone(), counter_staging_buffer);
    }
}
