use std::collections::HashMap;

use bevy::{
    ecs::system::SystemParam,
    prelude::{Commands, Entity, Query, Res, Resource},
    render::renderer::{RenderDevice, RenderQueue},
};
use bevy_gpu_compute_core::{
    TypesSpec, wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion,
};

use crate::task::task_specification::{
    max_output_vector_lengths::MaxOutputLengths, task_specification::ComputeTaskSpecification,
};

use super::task::{
    events::GpuAcceleratedTaskCreatedEvent,
    task_commands::TaskCommands,
    task_components::{task::BevyGpuComputeTask, task_name::TaskName},
    task_specification::iteration_space::IterationSpace,
};

#[derive(SystemParam)]

pub struct BevyGpuCompute<'w, 's> {
    tasks: Query<'w, 's, (Entity, &'static TaskName)>,
    render_device: Res<'s, RenderDevice>,
    render_queue: Res<'s, RenderQueue>,
}

impl<'w, 's> BevyGpuCompute<'w, 's> {
    /// spawns all components needed for the task to run, and returns a TaskCommands object that can be used for altering or running the task
    pub fn create_task_from_rust_shader<ShaderModuleTypes: TypesSpec>(
        &mut self,
        name: &str,
        mut commands: &mut Commands,
        gpu: &RenderDevice,
        wgsl_shader_module: WgslShaderModuleUserPortion,
        iteration_space: IterationSpace,
        max_output_vector_lengths: MaxOutputLengths,
    ) -> TaskCommands {
        let task = BevyGpuComputeTask::new();
        let entity = {
            let entity = commands.spawn((task, TaskName::new(name))).id();
            entity
        };
        let task_spec = ComputeTaskSpecification::from_shader::<ShaderModuleTypes>(
            name,
            &mut commands,
            entity,
            &gpu,
            wgsl_shader_module,
            iteration_space,
            max_output_vector_lengths,
        );
        commands.entity(entity).insert(task_spec);
        let task_commands = TaskCommands::new(entity);
        self.tasks.insert(name.to_string(), task_commands.clone());
        commands.send_event(GpuAcceleratedTaskCreatedEvent {
            entity,
            task_name: name.to_string(),
        });
        task_commands
    }
    pub fn task_exists(&self, name: &String) -> bool {
        self.tasks.contains_key(name)
    }
    pub fn task(&self, name: &str) -> &TaskCommands {
        if let Some(tc) = self.tasks.get(name) {
            &tc
        } else {
            panic!("task not found")
        }
    }
}
