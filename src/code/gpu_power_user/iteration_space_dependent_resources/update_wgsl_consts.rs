use std::cell::RefMut;

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{DetectChanges, Mut, Query, Ref, Res, ResMut, With},
};

use crate::code::{
    gpu_power_user::{outputs::output_spec::OutputSpecs, resources::GpuAcceleratedBevy},
    helpers::math::max_collisions::max_collisions,
};

use super::{
    iteration_space::IterationSpace,
    max_num_outputs_per_type::MaxNumGpuOutputItemsPerOutputType,
    workgroup_sizes::{NumGpuWorkgroupsRequired, WorkgroupSizes},
};

pub fn update_wgsl_consts(
    mut tasks: Query<
        (
            Ref<IterationSpace>,
            &OutputSpecs,
            &mut WorkgroupSizes,
            &mut MaxNumGpuOutputItemsPerOutputType,
            &mut NumGpuWorkgroupsRequired,
        ),
        With<GpuAcceleratedBevy>,
    >,
) {
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                iteration_space,
                output_types,
                mut workgroup_sizes,
                mut max_num_outputs_per_type,
                mut num_gpu_workgroups_required,
            )| {
                update_wgsl_params_single_task(
                    iteration_space,
                    output_types,
                    workgroup_sizes,
                    &mut max_num_outputs_per_type,
                    &mut num_gpu_workgroups_required,
                );
            },
        );
}

fn update_wgsl_params_single_task<'a>(
    iteration_space: Ref<IterationSpace>,
    output_types: &OutputSpecs,
    mut workgroup_sizes: Mut<'a, WorkgroupSizes>,
    max_num_outputs_per_type: &mut MaxNumGpuOutputItemsPerOutputType,
    num_gpu_workgroups_required: &mut NumGpuWorkgroupsRequired,
) {
    if iteration_space.x > 1 || iteration_space.y > 1 || iteration_space.z > 1 {
        if iteration_space.is_changed() || workgroup_sizes.is_changed() {
            if max_num_outputs_per_type.uses_callback {
                max_num_outputs_per_type.update_with_callback(
                    &*iteration_space,
                    // get all keys from this hashmap
                    output_types.specs.keys().collect::<Vec<_>>(),
                );
            }
            num_gpu_workgroups_required.x =
                (iteration_space.x as f32 / workgroup_sizes.x() as f32).ceil() as u32;
            num_gpu_workgroups_required.y =
                (iteration_space.y as f32 / workgroup_sizes.y() as f32).ceil() as u32;
            num_gpu_workgroups_required.z =
                (iteration_space.z as f32 / workgroup_sizes.z() as f32).ceil() as u32;
            if iteration_space.num_dimmensions() != workgroup_sizes.num_dimmensions() {
                *workgroup_sizes = WorkgroupSizes::from_iter_space(iteration_space);
            }
        }
    }
}
