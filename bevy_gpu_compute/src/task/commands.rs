use bevy::prelude::Entity;
use bevy_gpu_compute_core::{
    MaxOutputLengths, TypeErasedArrayInputData, TypeErasedConfigInputData,
};

use crate::prelude::IterationSpace;

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
impl std::fmt::Display for GpuTaskCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuTaskCommand::SetConfigInputs(_) => write!(f, "SetConfigInputs"),
            GpuTaskCommand::SetInputs(_) => write!(f, "SetInputs"),
            GpuTaskCommand::Mutate {
                iteration_space,
                max_output_lengths,
            } => write!(
                f,
                "Mutate {{ iteration_space: {:?}, max_output_lengths: {:?} }}",
                iteration_space, max_output_lengths
            ),
            GpuTaskCommand::Run => write!(f, "Run"),
        }
    }
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
