use std::collections::HashMap;

use bevy::prelude::{Commands, Resource};

use crate::task::task_specification::task_specification::ComputeTaskSpecification;

use super::task::{
    events::{
        GpuAcceleratedTaskCreatedEvent, GpuComputeTaskChangeEvent,
        IterationSpaceOrMaxOutVecLengthChangedEvent, WgslCodeChangedEvent,
    },
    inputs::input_vector_metadata_spec::InputVectorsMetadataSpec,
    outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
    task_commands::TaskCommands,
    task_components::{task::GpuAcceleratedBevyTask, task_name::TaskName},
    task_specification::iteration_space::IterationSpace,
    wgsl_code::WgslCode,
};

#[derive(Resource)]
pub struct GpuAcceleratedBevy {
    tasks: HashMap<String, TaskCommands>,
}
impl Default for GpuAcceleratedBevy {
    fn default() -> Self {
        GpuAcceleratedBevy {
            tasks: HashMap::new(),
        }
    }
}

impl GpuAcceleratedBevy {
    pub fn new() -> Self {
        GpuAcceleratedBevy {
            tasks: HashMap::new(),
        }
    }

    /// spawns all components needed for the task to run, and returns a TaskCommands object that can be used for altering or running the task
    pub fn create_task(
        &mut self,
        commands: &mut Commands,
        name: &String,
        spec: ComputeTaskSpecification,
    ) -> TaskCommands {
        let task = GpuAcceleratedBevyTask::new();
        let entity_commands = commands.spawn((task, spec, TaskName::new(name)));
        let entity = entity_commands.id();
        let task_commands = TaskCommands::new(entity);
        self.tasks.insert(name.clone(), task_commands.clone());
        commands.send_event(GpuAcceleratedTaskCreatedEvent {
            entity,
            task_name: name.clone(),
        });
        commands.send_event(IterationSpaceOrMaxOutVecLengthChangedEvent::new(entity));
        commands.send_event(WgslCodeChangedEvent::new(entity));
        task_commands
    }
    pub fn task_exists(&self, name: &String) -> bool {
        self.tasks.contains_key(name)
    }
    pub fn task(&self, name: &String) -> &TaskCommands {
        if let Some(tc) = self.tasks.get(name) {
            &tc
        } else {
            panic!("task not found")
        }
    }
}
