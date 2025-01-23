use std::collections::HashMap;

use bevy::{
    ecs::system::SystemParam,
    prelude::{Commands, DespawnRecursiveExt, Entity, Query, Res, Resource},
    render::renderer::{RenderDevice, RenderQueue},
};
use bevy_gpu_compute_core::{
    TypesSpec, wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion,
};

use crate::{
    prelude::IterationSpace,
    task::{
        task_components::{task::BevyGpuComputeTask, task_name::TaskName},
        task_specification::{
            max_output_vector_lengths::MaxOutputLengths,
            task_specification::ComputeTaskSpecification,
        },
    },
};

#[derive(SystemParam)]

pub struct BevyGpuComputeTaskDeleter<'w, 's> {
    commands: Commands<'w, 's>,
    tasks: Query<'w, 's, (Entity, &'static mut BevyGpuComputeTask)>,
}

impl<'w, 's> BevyGpuComputeTaskDeleter<'w, 's> {
    /// spawns all components needed for the task to run
    pub fn delete(&mut self, name: &str) {
        let (entity, _) = self
            .tasks
            .iter_mut()
            .find(|(_, task)| task.name() == name)
            .expect("Task not found");
        self.commands.entity(entity).despawn_recursive();
    }
}
