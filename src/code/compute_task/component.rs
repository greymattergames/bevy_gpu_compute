use bevy::prelude::{Component, Entity, EventWriter};

use super::{
    buffers::components::{
        GpuAcceleratedBevyBuffers, InputBuffers, OutputBuffers, OutputCountBuffers,
        OutputCountStagingBuffers, OutputStagingBuffers,
    },
    dispatch::create_bind_group::BindGroupComponent,
    inputs::{
        input_data::InputData,
        input_spec::{BlankInputVectorTypesSpec, InputVectorTypesSpec},
    },
    iteration_space_dependent_components::{
        iteration_space::IterationSpace,
        max_output_vector_lengths::MaxOutputVectorLengths,
        pipeline::cache::PipelineCache,
        workgroup_sizes::{NumGpuWorkgroupsRequired, WorkgroupSizes},
    },
    outputs::{
        misc_components::OutputCountsFromGpu,
        output_data::{OutputData, TypeErasedOutputData},
        output_spec::{BlankOutputVectorTypesSpec, OutputVectorTypesSpec},
    },
    resources::GpuAcceleratedBevy,
};
#[derive(Component)]
pub struct TaskRunId(pub u128);
impl Default for TaskRunId {
    fn default() -> Self {
        TaskRunId(0)
    }
}
#[derive(Component)]
pub struct TaskName(String);
impl Default for TaskName {
    fn default() -> Self {
        TaskName("unitialized task".to_string())
    }
}

impl TaskName {
    pub fn new(name: &str) -> Self {
        TaskName(name.to_string())
    }
    pub fn get(&self) -> &str {
        &self.0
    }
}

#[derive(Component)]
#[require(
    TaskName,
    TaskRunId,
    GpuAcceleratedBevy,
    IterationSpace,
    WorkgroupSizes,
    MaxOutputVectorLengths,
    NumGpuWorkgroupsRequired,
    PipelineCache,
    GpuAcceleratedBevyBuffers,
    BindGroupComponent,
    InputData<BlankInputVectorTypesSpec>,
    TypeErasedOutputData,
    OutputCountsFromGpu,
)]

pub struct GpuComputeTask<I: InputVectorTypesSpec, O: OutputVectorTypesSpec> {
    // input_spec: I,
    // output_spec: O,
    entity: Option<Entity>,
    phantom: std::marker::PhantomData<(I, O)>,
}

impl<I: InputVectorTypesSpec, O: OutputVectorTypesSpec> GpuComputeTask<I, O> {
    // pub fn new(input_spec: I, output_spec: O) -> Self {
    // Self {
    // input_spec,
    // output_spec,
    // }
    // }
    pub fn new() -> Self {
        Self {
            // input_spec,
            // output_spec,
            entity: None,
            phantom: std::marker::PhantomData,
        }
    }
    pub fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }
}
