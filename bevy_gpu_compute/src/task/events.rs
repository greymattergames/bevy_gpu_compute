use bevy::prelude::{Entity, Event};

#[derive(Event)]
pub struct GpuAcceleratedTaskCreatedEvent {
    pub entity: Entity,
    pub task_name: String,
}

pub trait GpuComputeTaskChangeEvent {
    fn new(entity: Entity) -> Self;
    fn entity(&self) -> Entity;
}
#[derive(Event)]
pub struct GpuComputeTaskSuccessEvent {
    pub id: u128,
}
