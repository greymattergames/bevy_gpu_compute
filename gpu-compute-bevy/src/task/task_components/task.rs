use bevy::prelude::{Component, Entity};
use shared::misc_types::BlankTypesSpec;

use crate::task::{
    buffers::components::{
        InputBuffers, OutputBuffers, OutputCountBuffers, OutputCountStagingBuffers,
        OutputStagingBuffers,
    },
    compute_pipeline::cache::PipelineLruCache,
    dispatch::create_bind_group::BindGroupComponent,
    inputs::input_data::InputData,
    outputs::definitions::{
        gpu_output_counts::GpuOutputCounts, type_erased_output_data::TypeErasedOutputData,
    },
    task_specification::{
        gpu_workgroup_sizes::GpuWorkgroupSizes, gpu_workgroup_space::GpuWorkgroupSpace,
        iteration_space::IterationSpace, task_specification::ComputeTaskSpecification,
    },
};

use super::{
    task_max_output_bytes::TaskMaxOutputBytes, task_name::TaskName, task_run_id::TaskRunId,
};

/**
A task can only run once per run of the GpuAcceleratedBevyRunTaskSet system set
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

    BindGroupComponent,
    InputData<BlankTypesSpec>,
    TypeErasedOutputData,
    GpuOutputCounts,
)]

pub struct GpuAcceleratedBevyTask
// <I: InputVectorTypesSpec, O: OutputVectorTypesSpec>
{
    entity: Option<Entity>,
    // phantom: std::marker::PhantomData<(I, O)>,
}

impl GpuAcceleratedBevyTask
// <I, O>
{
    pub fn new() -> Self {
        Self {
            entity: None,
            // phantom: std::marker::PhantomData,
        }
    }
    pub fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }
}
