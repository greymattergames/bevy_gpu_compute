use bevy::prelude::{Component, Entity, EventWriter};

use super::{
    buffers::components::{
        GpuAcceleratedBevyBuffers, InputBuffers, OutputBuffers, OutputCountBuffers,
        OutputCountStagingBuffers, OutputStagingBuffers,
    },
    inputs::{
        input_data::InputData,
        input_spec::{BlankInputVectorTypesSpec, InputVectorTypesSpec},
    },
    iteration_space_dependent_resources::{
        iteration_space::IterationSpace,
        max_num_outputs_per_type::MaxNumGpuOutputItemsPerOutputType,
        pipeline::cache::PipelineCache,
        workgroup_sizes::{NumGpuWorkgroupsRequired, WorkgroupSizes},
    },
    outputs::{
        misc_components::OutputCountsFromGpu,
        output_data::OutputData,
        output_spec::{BlankOutputVectorTypesSpec, OutputVectorTypesSpec},
    },
    resources::GpuAcceleratedBevy,
    single_batch::create_bind_group::BindGroupComponent,
};

#[derive(Component)]
pub struct TaskName(pub String);
impl Default for TaskName {
    fn default() -> Self {
        TaskName("unitialized task".to_string())
    }
}

#[derive(Component)]
#[require(
    TaskName,
    GpuAcceleratedBevy,
    IterationSpace,
    WorkgroupSizes,
    MaxNumGpuOutputItemsPerOutputType,
    NumGpuWorkgroupsRequired,
    PipelineCache,
    GpuAcceleratedBevyBuffers,
    BindGroupComponent,
    InputData<BlankInputVectorTypesSpec>,
    OutputData<BlankOutputVectorTypesSpec>,
    OutputCountsFromGpu,
)]

pub struct GpuComputeTask<I: InputVectorTypesSpec, O: OutputVectorTypesSpec> {
    input_spec: I,
    output_spec: O,
}

impl<I: InputVectorTypesSpec, O: OutputVectorTypesSpec> GpuComputeTask<I, O> {
    pub fn new(input_spec: I, output_spec: O) -> Self {
        Self {
            input_spec,
            output_spec,
        }
    }
}
