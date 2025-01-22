use bevy::{
    ecs::schedule::NodeConfigs,
    prelude::{IntoSystemConfigs, SystemSet},
};

use crate::task::{
    buffers::create_config_input_buffers::create_config_input_buffers,
    compute_pipeline::update_on_pipeline_const_change::update_pipelines_on_pipeline_const_change,
    inputs::array_type::handle_input_data_change::handle_input_data_change,
};

use super::task::{
    buffers::{
        create_input_buffers::create_input_buffers, create_output_buffers::create_output_buffers,
    },
    dispatch::{create_bind_group::create_bind_groups, dispatch_to_gpu::dispatch_to_gpu},
    outputs::{
        read_gpu_output_counts::read_gpu_output_counts,
        read_gpu_task_outputs::read_gpu_task_outputs,
    },
    verify_enough_memory::verify_have_enough_memory,
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct BevyGpuComputeRunTaskSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct BevyGpuComputeRespondToTaskMutSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct BevyGpuComputeRespondToInputsMutSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]

struct BevyGpuComputeDispatchSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct BevyGpuComputeReadSet;

pub fn compose_task_runner_systems()
-> NodeConfigs<Box<dyn bevy::prelude::System<In = (), Out = ()>>> {
    let respond_to_new_inputs = (
        handle_input_data_change,
        create_input_buffers,
        create_config_input_buffers,
    )
        .in_set(BevyGpuComputeRespondToInputsMutSet);
    let respond_to_task_alteration = (
        update_pipelines_on_pipeline_const_change,
        create_output_buffers,
        verify_have_enough_memory,
    )
        .in_set(BevyGpuComputeRespondToTaskMutSet);
    let dispatch = (create_bind_groups, dispatch_to_gpu)
        .chain()
        .in_set(BevyGpuComputeDispatchSet);
    let read = (read_gpu_output_counts, read_gpu_task_outputs)
        .chain()
        .in_set(BevyGpuComputeReadSet);
    let run_task_set = (
        respond_to_new_inputs,
        respond_to_task_alteration,
        dispatch,
        read,
    )
        .chain()
        .in_set(BevyGpuComputeRunTaskSet);
    return run_task_set;
}
