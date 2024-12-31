use std::{any::Any, collections::HashMap};

use bevy::{
    prelude::{Commands, DespawnRecursiveExt, Entity, Event, Ref, ResMut, Resource},
    reflect::Tuple,
    state::commands,
};

use super::{
    compute_task::{
        component::GpuComputeTask,
        inputs::{input_data::InputData, input_specs::InputSpecs},
        iteration_space_dependent_resources::{
            iteration_space::IterationSpace,
            max_num_outputs_per_type::MaxNumGpuOutputItemsPerOutputType,
        },
        outputs::output_spec::OutputSpecs,
        wgsl_code::WgslCode,
    },
    to_vec_tuple::ToVecTuple,
};

#[derive(Resource)]
pub struct GpuCompute {
    task_run_counter: u128,
    tasks: HashMap<String, Entity>,
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
    type InType: Tuple;
    type OutType: Tuple;
}
#[derive(Event)]
pub struct GpuTaskFinishedEvent<T: GpuComputeBevyTaskType> {
    pub results: T::OutType,
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
    pub fn register_task<T: GpuComputeBevyTaskType>(
        commands: &mut Commands,
        name: &String,
        iteration_space: IterationSpace,
        wgsl: WgslCode,
        inputs: InputSpecs,
        outputs: OutputSpecs,
        max_num_outputs: MaxNumGpuOutputItemsPerOutputType,
    ) -> String {
        let entity_commands = commands.spawn((
            GpuComputeTask::new(name.clone()),
            iteration_space,
            wgsl,
            inputs,
            outputs,
            max_num_outputs,
        ));
        let entity = entity_commands.id();
        name.clone()
    }
    pub fn delete_task<T: GpuComputeBevyTaskType>(&mut self, name: String) {
        let entity = self.tasks.get(&name).unwrap().clone();
        self.tasks.remove(&name);
        self.to_delete.push(entity);
    }
    pub fn alter_task<T: GpuComputeBevyTaskType>(name: String) {
        todo!("alter_task")
    }
    /// registers the input data to run in the next round, returns a unique id to identify the run
    pub fn run<T: GpuComputeBevyTaskType, U: ToVecTuple<T::InType>>(
        &mut self,
        task_name: String,
        inputs: U,
        commands: &mut Commands,
    ) -> u128 {
        self.task_run_counter += 1;
        let mut entity = commands.entity(self.tasks.get(&task_name).unwrap().clone())
        let input_data = entity.entry::<InputData>();
        input_data.and_modify(|data| {
            data.data = inputs.to_vec_tuple();
        })
        // Otherwise insert a default value
        .or_insert(Level(0));
        self.task_run_counter
    }
}

//todo, register this system
pub fn delete_tasks_system(commands: &mut Commands, gpu_compute: ResMut<GpuCompute>) {
    for entity in gpu_compute.to_delete.iter() {
        commands.entity(*entity).despawn_recursive();
    }
}

pub struct GpuComputeTaskMutator {
    //todo
}
