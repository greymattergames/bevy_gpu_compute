use bevy::prelude::{DetectChanges, Res, ResMut};

use crate::code::{
    gpu_power_user::{
        population_dependent_resources::resources::{IterationSpace, WorkgroupSizes},
        resources::MaxDetectableCollisionsScale,
    },
    helpers::math::max_collisions::max_collisions,
};

use super::resources::{MaxNumGpuOutputItemsPerOutputType, NumGpuWorkgroupsRequired};

pub fn update_wgsl_consts(
    iteration_space: Res<IterationSpace>,
    output_types: Res<GpuOutputTypes>,
    mut workgroup_sizes: ResMut<WorkgroupSizes>,
    mut max_num_outputs_per_type: ResMut<MaxNumGpuOutputItemsPerOutputType>,
    mut num_gpu_workgroups_required: ResMut<NumGpuWorkgroupsRequired>,
) {
    if iteration_space.x > 1 || iteration_space.y > 1 || iteration_space.z > 1 {
        if iteration_space.is_changed() || workgroup_sizes.is_changed() {
            if max_num_outputs_per_type.uses_callback {
                max_num_outputs_per_type.update_with_callback(
                    &*iteration_space,
                    output_types.get_output_variable_names(),
                );
            }
            num_gpu_workgroups_required.x =
                (iteration_space.x as f32 / workgroup_sizes.x() as f32).ceil() as usize;
            num_gpu_workgroups_required.y =
                (iteration_space.y as f32 / workgroup_sizes.y() as f32).ceil() as usize;
            num_gpu_workgroups_required.z =
                (iteration_space.z as f32 / workgroup_sizes.z() as f32).ceil() as usize;
            if iteration_space.num_dimmensions() != workgroup_sizes.num_dimmensions() {
                *workgroup_sizes = WorkgroupSizes::from_iter_space(*iteration_space);
            }
        }
    }
}
