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
pub struct InputDataChangeEvent {
    entity: Entity,
    pub lengths: [Option<usize>; 6],
}
impl InputDataChangeEvent {
    pub fn new(entity: Entity, lengths: [Option<usize>; 6]) -> Self {
        InputDataChangeEvent { entity, lengths }
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
}
#[derive(Event)]
pub struct ConfigInputDataChangeEvent {
    entity: Entity,
}
impl ConfigInputDataChangeEvent {
    pub fn new(entity: Entity) -> Self {
        ConfigInputDataChangeEvent { entity }
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Event)]
pub struct IterSpaceOrOutputSizesChangedEvent {
    entity: Entity,
}
impl GpuComputeTaskChangeEvent for IterSpaceOrOutputSizesChangedEvent {
    fn new(entity: Entity) -> Self {
        IterSpaceOrOutputSizesChangedEvent { entity }
    }
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Event)]
pub struct GpuComputeTaskSuccessEvent {
    pub id: u128,
}
