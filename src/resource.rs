use std::collections::HashMap;

use bevy::prelude::{Commands, Resource};

use super::task::{
    events::{
        GpuAcceleratedTaskCreatedEvent, GpuComputeTaskChangeEvent,
        IterationSpaceOrMaxOutVecLengthChangedEvent, WgslCodeChangedEvent,
    },
    inputs::input_vector_metadata_spec::InputVectorMetadataSpec,
    iteration_space::iteration_space::IterationSpace,
    outputs::definitions::{
        max_output_vector_lengths::MaxOutputVectorLengths,
        output_vector_metadata_spec::OutputVectorMetadataSpec,
    },
    task_commands::TaskCommands,
    task_components::{task::GpuAcceleratedBevyTask, task_name::TaskName},
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
        iteration_space: IterationSpace,
        wgsl: WgslCode,
        input_vectors_metadata_spec: InputVectorMetadataSpec,
        output_vectors_metadata_spec: OutputVectorMetadataSpec,
        max_output_vectors_lengths: MaxOutputVectorLengths,
    ) -> TaskCommands {
        let task = GpuAcceleratedBevyTask::new();
        let entity_commands = commands.spawn((
            task,
            TaskName::new(name),
            iteration_space,
            wgsl,
            input_vectors_metadata_spec,
            output_vectors_metadata_spec,
            max_output_vectors_lengths,
        ));
        let entity = entity_commands.id();
        let task_commands = TaskCommands::new(entity);
        self.tasks.insert(name.clone(), task_commands.clone());
        commands.send_event(GpuAcceleratedTaskCreatedEvent {
            entity,
            task_name: name.clone(),
            input_vector_metadata_spec: input_vectors_metadata_spec.clone(),
            output_vector_metadata_spec: output_vectors_metadata_spec.clone(),
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
