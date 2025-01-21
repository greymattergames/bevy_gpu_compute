use std::collections::HashMap;

use bevy::prelude::{Commands, Entity};

use crate::task::{
    events::{GpuComputeTaskChangeEvent, InputDataChangeEvent, IterSpaceOrOutputSizesChangedEvent},
    task_components::task_max_output_bytes::TaskMaxOutputBytes,
};

use super::{
    derived_spec::ComputeTaskDerivedSpec, gpu_workgroup_sizes::GpuWorkgroupSizes,
    gpu_workgroup_space::GpuWorkgroupSpace, immutable_spec::ComputeTaskImmutableSpec,
    input_array_lengths::ComputeTaskInputArrayLengths, iteration_space::IterationSpace,
    max_output_vector_lengths::MaxOutputLengths,
};

#[derive(Default, Debug)]
pub struct ComputeTaskMutableSpec {
    iteration_space: IterationSpace,
    input_array_lengths: ComputeTaskInputArrayLengths,
    output_array_lengths: MaxOutputLengths,
    iter_space_and_out_lengths_version: u64,
}

impl ComputeTaskMutableSpec {
    pub fn new(
        iteration_space: IterationSpace,
        input_array_lengths: ComputeTaskInputArrayLengths,
        output_array_lengths: MaxOutputLengths,
        derived: &mut ComputeTaskDerivedSpec,
        immutable: &ComputeTaskImmutableSpec,
        mut commands: &mut Commands,
        entity: Entity,
    ) -> Self {
        let mut mutable = ComputeTaskMutableSpec {
            iteration_space,
            input_array_lengths,
            output_array_lengths,
            iter_space_and_out_lengths_version: 0,
        };
        mutable.update_on_iter_space_or_max_output_lengths_change(
            derived,
            immutable,
            &mut commands,
            entity,
        );
        mutable
    }

    pub fn iteration_space(&self) -> &IterationSpace {
        &self.iteration_space
    }
    pub fn input_array_lengths(&self) -> &ComputeTaskInputArrayLengths {
        &self.input_array_lengths
    }
    pub fn output_array_lengths(&self) -> &MaxOutputLengths {
        &self.output_array_lengths
    }
    pub fn iter_space_and_out_lengths_version(&self) -> u64 {
        self.iter_space_and_out_lengths_version
    }

    /// one of each event type maximum is sent per call, so this is more efficient than updating each field individually
    /// If a parameter is None then the existing value is retained
    pub fn multiple(
        &mut self,
        iteration_space: Option<IterationSpace>,
        input_array_lengths: Option<ComputeTaskInputArrayLengths>,
        output_array_lengths: Option<MaxOutputLengths>,
        immutable: &ComputeTaskImmutableSpec,
        mut derived: &mut ComputeTaskDerivedSpec,
        mut commands: &mut Commands,
        entity: Entity,
    ) {
        let iter_or_outputs_changed = iteration_space.is_some() || output_array_lengths.is_some();
        if let Some(iter_space) = iteration_space {
            // ensure that the number of dimmensions has not been changed
            assert!(
                iter_space.num_dimmensions() == self.iteration_space.num_dimmensions(),
                "The number of dimmensions cannot be changed after creating the task. Currently the iteration space for this task is {:?}, but you are trying to change it to be {:?}. For example: an iteration space of x = 30, y = 20 and z = 1 has 2 dimmensions, and an iteration space of x = 30, y=1, z=1 has 1 dimmension.",
                self.iteration_space.num_dimmensions() as usize,
                iter_space.num_dimmensions() as usize
            );
            self.iteration_space = iter_space;
        }
        if let Some(input_lengths) = input_array_lengths {
            self.input_array_lengths = input_lengths;
        }
        if let Some(output_lengths) = output_array_lengths {
            self.output_array_lengths = output_lengths;
        }
        if iter_or_outputs_changed {
            self.update_on_iter_space_or_max_output_lengths_change(
                &mut derived,
                &immutable,
                &mut commands,
                entity,
            );
        }
    }
    fn update_on_iter_space_or_max_output_lengths_change(
        &mut self,
        mut derived: &mut ComputeTaskDerivedSpec,
        immutable: &ComputeTaskImmutableSpec,
        mut commands: &mut Commands,
        entity: Entity,
    ) {
        self.iter_space_and_out_lengths_version += 1;
        // update task max output bytes
        derived._lib_only_set_task_max_output_bytes(TaskMaxOutputBytes::from_max_lengths_and_spec(
            &self.output_array_lengths,
            &immutable.output_vectors_metadata_spec(),
        ));
        let mut wg_sizes = derived.workgroup_sizes().clone();
        // update workgroup sizes
        if self.iteration_space.num_dimmensions() as usize != wg_sizes.num_dimmensions() {
            wg_sizes = GpuWorkgroupSizes::from_iter_space(&self.iteration_space);
            derived._lib_only_set_workgroup_sizes(wg_sizes.clone());
        }
        derived._lib_only_set_gpu_workgroup_space(
            GpuWorkgroupSpace::from_iter_space_and_wrkgrp_sizes(&self.iteration_space, &wg_sizes),
        );
        commands.send_event(IterSpaceOrOutputSizesChangedEvent::new(entity));
    }
}
