use std::{any::Any, collections::HashMap};

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{
        Commands, CommandsStatesExt, Component, DespawnRecursiveExt, Entity, Event, EventReader,
        EventWriter, Ref, ResMut, Resource,
    },
    reflect::{PartialReflect, Tuple},
    state::commands,
};
use futures::future::Either;

use super::{
    compute_task::{
        component::{GpuComputeTask, TaskName, TaskRunId},
        events::{
            GpuComputeTaskChangeEvent, InputDataChangeEvent, IterationSpaceChangedEvent,
            MaxOutputVectorLengthsChangedEvent, WgslCodeChangedEvent,
        },
        inputs::{
            input_data::InputData, input_metadata_spec::InputVectorMetadataSpec,
            input_spec::InputVectorTypesSpec,
        },
        iteration_space_dependent_components::{
            iteration_space::IterationSpace, max_output_vector_lengths::MaxOutputVectorLengths,
        },
        outputs::{
            output_data::OutputData, output_metadata_spec::OutputVectorMetadataSpec,
            output_spec::OutputVectorTypesSpec,
        },
        task_commands::TaskCommands,
        wgsl_code::WgslCode,
    },
    to_vec_tuple::ToVecTuple,
};

#[derive(Resource)]
pub struct GpuCompute {
    task_run_counter: u128,
    tasks: HashMap<String, TaskCommands>,
    to_delete: Vec<Entity>,
}
impl Default for GpuCompute {
    fn default() -> Self {
        GpuCompute {
            task_run_counter: 0,
            tasks: HashMap::new(),
            to_delete: Vec::new(),
        }
    }
}

pub trait GpuComputeBevyTaskType {
    type InType: Component + InputVectorTypesSpec + 'static + Send + Sync;
    type OutType: OutputVectorTypesSpec + 'static + Send + Sync;
}
impl GpuCompute {
    pub fn new() -> Self {
        GpuCompute {
            task_run_counter: 0,
            tasks: HashMap::new(),
            to_delete: Vec::new(),
        }
    }

    /// spawns all components needed for the task to run, and returns the name given to the task
    pub fn create_task<T: GpuComputeBevyTaskType>(
        &mut self,
        commands: &mut Commands,
        name: &String,
        iteration_space: IterationSpace,
        wgsl: WgslCode,
        // input_vector_types_spec: T::InType,
        // output_vector_types_spec: T::OutType,
        input_vector_metadata_spec: InputVectorMetadataSpec,
        output_vector_metadata_spec: OutputVectorMetadataSpec,
        max_num_outputs: MaxOutputVectorLengths,
    ) -> TaskCommands {
        let task = GpuComputeTask::<T::InType, T::OutType>::new();
        let entity_commands = commands.spawn((
            task,
            TaskName::new(name),
            iteration_space,
            wgsl,
            input_vector_metadata_spec,
            output_vector_metadata_spec,
            max_num_outputs,
        ));
        let entity = entity_commands.id();
        let task_commands = TaskCommands::new(entity);
        self.tasks.insert(name.clone(), task_commands.clone());
        task_commands
    }
    pub fn task(&self, name: &String) -> &TaskCommands {
        if let Some(tc) = self.tasks.get(name) {
            &tc
        } else {
            panic!("task not found")
        }
    }
}

#[derive(Resource)]

pub struct GpuAcceleratedBevyRunIds {
    last_id: u128,
}
impl Default for GpuAcceleratedBevyRunIds {
    fn default() -> Self {
        GpuAcceleratedBevyRunIds { last_id: 0 }
    }
}
impl GpuAcceleratedBevyRunIds {
    pub fn get_next(&mut self) -> u128 {
        self.last_id += 1;
        self.last_id
    }
}
