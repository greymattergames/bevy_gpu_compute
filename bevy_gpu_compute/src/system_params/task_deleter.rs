
use bevy::{
    ecs::system::SystemParam,
    prelude::{Commands, DespawnRecursiveExt, Entity, Query},
};

use crate::task::task_components::task::BevyGpuComputeTask;

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
