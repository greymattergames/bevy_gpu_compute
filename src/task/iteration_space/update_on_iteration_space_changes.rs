use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{DetectChanges, EventReader, Mut, Query, Ref},
};

use crate::task::{
    events::{GpuComputeTaskChangeEvent, IterationSpaceOrMaxOutVecLengthChangedEvent},
    outputs::definitions::{
        max_output_vector_lengths::MaxOutputVectorLengths,
        output_vector_metadata_spec::OutputVectorMetadataSpec,
    },
    task_components::task_max_output_bytes::TaskMaxOutputBytes,
};

use super::{
    gpu_workgroup_sizes::GpuWorkgroupSizes, gpu_workgroup_space::GpuWorkgroupSpace,
    iteration_space::IterationSpace,
};

/// updates MaxOutputVectorLengths, WorkgroupSizes, and NumGpuWorkgroupsRequired when an iteration space is changed
/// TODO Make this immediate and synchronous
pub fn update_gpu_params_on_iteration_space_or_max_output_lengths_change(
    mut tasks: Query<(
        Ref<IterationSpace>,
        &mut GpuWorkgroupSizes,
        &mut MaxOutputVectorLengths,
        &mut GpuWorkgroupSpace,
        &OutputVectorMetadataSpec,
        &mut TaskMaxOutputBytes,
    )>,
    mut change_event_reader: EventReader<IterationSpaceOrMaxOutVecLengthChangedEvent>,
) {
    for (ev, _) in change_event_reader
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        let task = tasks.get_mut(ev.entity().clone());
        if let Ok((
            iteration_space,
            mut workgroup_sizes,
            mut max_output_vector_lengths,
            mut num_gpu_workgroups_required,
            output_vector_metadata_spec,
            mut task_max_output_bytes,
        )) = task
        {
            update_gpu_params_single_task(
                iteration_space,
                workgroup_sizes,
                &mut max_output_vector_lengths,
                &mut num_gpu_workgroups_required,
                output_vector_metadata_spec,
                &mut task_max_output_bytes,
            );
        }
    }
}
/// updates MaxOutputVectorLengths, WorkgroupSizes, and NumGpuWorkgroupsRequired for a single task
fn update_gpu_params_single_task<'a>(
    iteration_space: Ref<IterationSpace>,
    mut workgroup_sizes: Mut<'a, GpuWorkgroupSizes>,
    max_output_vector_lengths: &mut MaxOutputVectorLengths,
    num_gpu_workgroups_required: &mut GpuWorkgroupSpace,
    output_vector_metadata_spec: &OutputVectorMetadataSpec,
    task_max_output_bytes: &mut TaskMaxOutputBytes,
) {
    if iteration_space.x() > 1 || iteration_space.y() > 1 || iteration_space.z() > 1 {
        if iteration_space.is_changed() || workgroup_sizes.is_changed() {
            // update task max output bytes
            *task_max_output_bytes = TaskMaxOutputBytes::from_max_lengths_and_spec(
                max_output_vector_lengths,
                output_vector_metadata_spec,
            );
            // update workgroup sizes
            if iteration_space.num_dimmensions() != workgroup_sizes.num_dimmensions() {
                *workgroup_sizes = GpuWorkgroupSizes::from_iter_space(&iteration_space);
            }
            // update num workgroups required
            num_gpu_workgroups_required.x =
                (iteration_space.x() as f32 / workgroup_sizes.x() as f32).ceil() as u32;
            num_gpu_workgroups_required.y =
                (iteration_space.y() as f32 / workgroup_sizes.y() as f32).ceil() as u32;
            num_gpu_workgroups_required.z =
                (iteration_space.z() as f32 / workgroup_sizes.z() as f32).ceil() as u32;
        }
    }
}
