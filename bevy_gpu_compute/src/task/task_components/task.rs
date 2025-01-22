use bevy::prelude::{Component, Entity};
use bevy_gpu_compute_core::BlankTypesSpec;

use crate::{
    prelude::ConfigInputData,
    task::{
        buffers::components::{
            ConfigInputBuffers, InputBuffers, OutputBuffers, OutputCountBuffers,
            OutputCountStagingBuffers, OutputStagingBuffers,
        },
        compute_pipeline::cache::PipelineLruCache,
        dispatch::create_bind_group::BindGroupComponent,
        inputs::array_type::input_data::InputData,
        outputs::definitions::{
            gpu_output_counts::GpuOutputCounts, type_erased_output_data::TypeErasedOutputData,
        },
        task_specification::task_specification::ComputeTaskSpecification,
    },
};

use super::{task_name::TaskName, task_run_id::TaskRunId};

/**
A task can only run once per run of the BevyGpuComputeRunTaskSet system set
By default this means once per frame
*/

#[derive(Component)]
#[require(
    TaskName,
    TaskRunId,
    ComputeTaskSpecification,
    PipelineLruCache,
    // buffers
    OutputBuffers,
    OutputCountBuffers,
    OutputStagingBuffers,
    OutputCountStagingBuffers,
    InputBuffers,
    ConfigInputBuffers,
    // other stuff
    BindGroupComponent,
    ConfigInputData<BlankTypesSpec>,
    InputData<BlankTypesSpec>,
    TypeErasedOutputData,
    GpuOutputCounts,
)]

pub struct BevyGpuComputeTask {}

impl BevyGpuComputeTask {
    pub fn new() -> Self {
        Self {}
    }
}
