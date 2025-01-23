use bevy::{
    gizmos::config,
    log,
    prelude::{Commands, DespawnRecursiveExt, Entity, Mut, Query, ResMut},
    render::renderer::{RenderDevice, RenderQueue},
};
use bevy_gpu_compute_core::TypesSpec;

use crate::{
    prelude::{
        ComputeTaskSpecification, ConfigInputData, InputData, IterationSpace, MaxOutputLengths,
    },
    run_ids::BevyGpuComputeRunIds,
    task::{
        buffers::components::{
            ConfigInputBuffers, InputBuffers, OutputBuffers, OutputCountBuffers,
            OutputCountStagingBuffers, OutputStagingBuffers,
        },
        inputs::{
            array_type::{
                input_data::InputDataTrait, input_vector_metadata_spec::InputVectorsMetadataSpec,
            },
            config_type::config_input_metadata_spec::ConfigInputsMetadataSpec,
        },
        outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
        task_components::{task::BevyGpuComputeTask, task_name::TaskName},
    },
};

use super::{
    inputs::{
        array_type::type_erased_input_data::TypeErasedInputData,
        config_type::type_erased_config_input_data::TypeErasedConfigInputData,
    },
    outputs::definitions::output_data::OutputData,
    task_specification::input_array_lengths::ComputeTaskInputArrayLengths,
};

pub struct GpuTaskCommands {
    entity: Entity,
    pub commands: Vec<GpuTaskCommand>,
}

pub enum GpuTaskCommand {
    SetConfigInputs(Box<TypeErasedConfigInputData>),
    SetInputs {
        data: Box<TypeErasedInputData>,
        lengths: ComputeTaskInputArrayLengths,
    },
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
    pub fn set_config_inputs<T: TypesSpec + 'static + Send + Sync>(
        mut self,
        inputs: ConfigInputData<T>,
    ) -> Self {
        self.commands.push(GpuTaskCommand::SetConfigInputs(Box::new(
            TypeErasedConfigInputData::new(inputs),
        )));
        self
    }

    /// This queues a mutation of the task. You still MUST call `GpuTaskRunner::run_commands` for this to take effect.
    pub fn set_inputs<T: TypesSpec + Send + Sync + 'static>(mut self, data: InputData<T>) -> Self {
        let lengths = data.lengths();
        self.commands.push(GpuTaskCommand::SetInputs {
            data: Box::new(TypeErasedInputData::new(data)),
            lengths: ComputeTaskInputArrayLengths { by_index: lengths },
        });
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
