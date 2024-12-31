use std::cell::RefMut;

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{DetectChanges, Entity, Event, EventReader, Mut, Query, Ref, Res, ResMut, With},
};

use crate::code::{
    compute_task::{
        events::{GpuComputeTaskChangeEvent, IterationSpaceChangedEvent},
        outputs::output_spec::OutputVectorTypesSpec,
        resources::GpuAcceleratedBevy,
    },
    helpers::math::max_collisions::max_collisions,
};

use super::{
    iteration_space::IterationSpace,
    max_output_vector_lengths::MaxOutputVectorLengths,
    workgroup_sizes::{NumGpuWorkgroupsRequired, WorkgroupSizes},
};

pub fn update_wgsl_params(
    mut tasks: Query<
        (
            Ref<IterationSpace>,
            &mut WorkgroupSizes,
            &mut MaxOutputVectorLengths,
            &mut NumGpuWorkgroupsRequired,
        ),
        With<GpuAcceleratedBevy>,
    >,
    mut iteration_space_changed_event_reader: EventReader<IterationSpaceChangedEvent>,
) {
    for (ev, _) in iteration_space_changed_event_reader
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        let task = tasks.get_mut(ev.entity().clone());
        if let Ok((
            iteration_space,
            mut workgroup_sizes,
            mut max_num_outputs_per_type,
            mut num_gpu_workgroups_required,
        )) = task
        {
            update_wgsl_params_single_task(
                iteration_space,
                workgroup_sizes,
                &mut max_num_outputs_per_type,
                &mut num_gpu_workgroups_required,
            );
        }
    }
}

fn update_wgsl_params_single_task<'a>(
    iteration_space: Ref<IterationSpace>,
    mut workgroup_sizes: Mut<'a, WorkgroupSizes>,
    max_num_outputs_per_type: &mut MaxOutputVectorLengths,
    num_gpu_workgroups_required: &mut NumGpuWorkgroupsRequired,
) {
    if iteration_space.x > 1 || iteration_space.y > 1 || iteration_space.z > 1 {
        if iteration_space.is_changed() || workgroup_sizes.is_changed() {
            // update max num outputs per type
            if max_num_outputs_per_type.uses_callback {
                max_num_outputs_per_type.update_with_callback(
                    &*iteration_space,
                    // get all keys from this hashmap
                );
            }
            // update workgroup sizes
            if iteration_space.num_dimmensions() != workgroup_sizes.num_dimmensions() {
                *workgroup_sizes = WorkgroupSizes::from_iter_space(&iteration_space);
            }
            // update num workgroups required
            num_gpu_workgroups_required.x =
                (iteration_space.x as f32 / workgroup_sizes.x() as f32).ceil() as u32;
            num_gpu_workgroups_required.y =
                (iteration_space.y as f32 / workgroup_sizes.y() as f32).ceil() as u32;
            num_gpu_workgroups_required.z =
                (iteration_space.z as f32 / workgroup_sizes.z() as f32).ceil() as u32;
        }
    }
}
