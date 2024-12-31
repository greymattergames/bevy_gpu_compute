use bevy::prelude::{Component, Entity, EventWriter};

use super::{
    buffers::components::{
        GpuAcceleratedBevyBuffers, InputBuffers, OutputBuffers, OutputCountBuffers,
        OutputCountStagingBuffers, OutputStagingBuffers,
    },
    events::ComputeTaskNameChangeEvent,
    inputs::{input_data::InputData, input_specs::InputSpecs},
    iteration_space_dependent_resources::{
        iteration_space::IterationSpace,
        max_num_outputs_per_type::MaxNumGpuOutputItemsPerOutputType,
        pipeline::cache::PipelineCache,
        workgroup_sizes::{NumGpuWorkgroupsRequired, WorkgroupSizes},
    },
    outputs::{
        latest_results_store::LatestResultsStore, misc_components::OutputCountsFromGpu,
        output_spec::OutputSpecs,
    },
    resources::GpuAcceleratedBevy,
    single_batch::create_bind_group::BindGroupComponent,
};

#[derive(Component)]
#[require(
    GpuAcceleratedBevy,
    InputSpecs,
    OutputSpecs,
    IterationSpace,
    WorkgroupSizes,
    MaxNumGpuOutputItemsPerOutputType,
    NumGpuWorkgroupsRequired,
    PipelineCache,
    GpuAcceleratedBevyBuffers,
    BindGroupComponent,
    InputData,
    OutputCountsFromGpu,
    LatestResultsStore
)]

pub struct GpuComputeTask {
    name: String,
}

impl GpuComputeTask {
    pub fn new(name: String) -> Self {
        Self { name }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn name_mut(
        &mut self,
        entity: Entity,
        mut event_writer: EventWriter<ComputeTaskNameChangeEvent>,
    ) -> &mut String {
        // send a change event
        event_writer.send(ComputeTaskNameChangeEvent {
            entity,
            new_name: self.name.clone(),
        });
        &mut self.name
    }
}
