use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{
        Commands, Component, DespawnRecursiveExt, Entity, Event, EventReader, EventWriter, Query,
        Ref, ResMut, Resource,
    },
    reflect::{PartialReflect, Tuple},
    state::commands,
};
use futures::future::Either;

use crate::code::manager_resource::GpuComputeBevyTaskType;

use super::{
    component::TaskRunId,
    events::{
        GpuComputeTaskChangeEvent, InputDataChangeEvent, IterationSpaceChangedEvent,
        MaxOutputVectorLengthsChangedEvent, WgslCodeChangedEvent,
    },
    inputs::input_data::InputData,
    iteration_space_dependent_components::{
        iteration_space::IterationSpace, max_output_vector_lengths::MaxOutputVectorLengths,
    },
    wgsl_code::WgslCode,
};
#[derive(Clone, Debug)]
pub struct TaskCommands {
    entity: Entity,
}
impl TaskCommands {
    pub fn new(entity: Entity) -> Self {
        TaskCommands { entity }
    }

    pub fn delete(&self, commands: &mut Commands) {
        commands.entity(self.entity).despawn_recursive();
    }

    pub fn set_iteration_space(
        &self,
        commands: &mut Commands,
        new_iteration_space: IterationSpace,
    ) {
        self.alter_task::<IterationSpaceChangedEvent, _>(commands, new_iteration_space);
    }
    pub fn set_max_output_vector_lengths(
        &self,
        commands: &mut Commands,
        new_max_output_vector_lengths: MaxOutputVectorLengths,
    ) {
        self.alter_task::<MaxOutputVectorLengthsChangedEvent, _>(
            commands,
            new_max_output_vector_lengths,
        );
    }
    pub fn set_wgsl_code(&self, commands: &mut Commands, new_wgsl_code: WgslCode) {
        self.alter_task::<WgslCodeChangedEvent, _>(commands, new_wgsl_code);
    }
    fn alter_task<E: Event + GpuComputeTaskChangeEvent, T: Component>(
        &self,
        commands: &mut Commands,
        new_component: T,
    ) {
        let mut entity_commands = commands.entity(self.entity);
        entity_commands.insert(new_component);
        commands.send_event(E::new(self.entity));
    }

    /// registers the input data to run in the next round, returns a unique id to identify the run
    pub fn run<T: GpuComputeBevyTaskType>(
        &self,
        commands: &mut Commands,
        inputs: InputData<T::InType>,
        mut task_run_ids: Query<&mut TaskRunId>,
    ) -> u128 {
        let mut entity_commands = commands.entity(self.entity);
        entity_commands.insert(inputs);
        commands.send_event(InputDataChangeEvent::new(self.entity));
        let mut tri = task_run_ids.get_mut(self.entity).unwrap();
        tri.0 += 1;
        tri.0
    }
}
