use bevy::{
    gizmos::config,
    log,
    prelude::{Commands, DespawnRecursiveExt, Entity, Mut, Query, ResMut},
    render::renderer::{RenderDevice, RenderQueue},
};
use bevy_gpu_compute_core::{TypeErasedArrayInputData, TypeErasedConfigInputData, TypesSpec};

use crate::{
    prelude::{ComputeTaskSpecification, IterationSpace, MaxOutputLengths},
    task::{
        buffers::components::{
            ConfigInputBuffers, InputBuffers, OutputBuffers, OutputCountBuffers,
            OutputCountStagingBuffers, OutputStagingBuffers,
        },
        inputs::{
            array_type::input_vector_metadata_spec::InputVectorsMetadataSpec,
            config_type::config_input_metadata_spec::ConfigInputsMetadataSpec,
        },
        outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
        task_components::{task::BevyGpuComputeTask, task_name::TaskName},
    },
};

pub struct GpuTaskCommands {
    entity: Entity,
    pub commands: Vec<GpuTaskCommand>,
}

pub enum GpuTaskCommand {
    SetConfigInputs(Box<TypeErasedConfigInputData>),
    SetInputs(Box<TypeErasedArrayInputData>),
    Mutate {
        iteration_space: Option<IterationSpace>,
        max_output_lengths: Option<MaxOutputLengths>,
    },
    Run,
}

impl GpuTaskCommands {
    pub fn new(entity: Entity) -> Self {
        GpuTaskCommands {
            entity,
            commands: Vec::new(),
        }
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
    /// This queues a mutation of the task. You still MUST call `GpuTaskRunner::run_commands` for this to take effect.
    pub fn set_config_inputs(mut self, inputs: TypeErasedConfigInputData) -> Self {
        self.commands
            .push(GpuTaskCommand::SetConfigInputs(Box::new(inputs)));
        self
    }

    /// This queues a mutation of the task. You still MUST call `GpuTaskRunner::run_commands` for this to take effect.
    pub fn set_inputs(mut self, data: TypeErasedArrayInputData) -> Self {
        self.commands
            .push(GpuTaskCommand::SetInputs(Box::new(data)));
        self
    }
    /// This queues a mutation of the task. You still MUST call `GpuTaskRunner::run_commands` for this to take effect.
    pub fn mutate(
        mut self,
        iteration_space: Option<IterationSpace>,
        max_output_lengths: Option<MaxOutputLengths>,
    ) -> Self {
        self.commands.push(GpuTaskCommand::Mutate {
            iteration_space,
            max_output_lengths,
        });
        self
    }

    /// This queues a run of the task. You still MUST call `GpuTaskRunner::run_commands` for this to take effect.
    pub fn run(mut self) -> Self {
        self.commands.push(GpuTaskCommand::Run);
        self
    }
}
