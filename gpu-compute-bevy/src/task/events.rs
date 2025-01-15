use bevy::prelude::{Entity, Event};

use super::{
    inputs::input_vector_metadata_spec::InputVectorsMetadataSpec,
    outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
};

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
}
impl GpuComputeTaskChangeEvent for InputDataChangeEvent {
    fn new(entity: Entity) -> Self {
        InputDataChangeEvent { entity }
    }
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Event)]
pub struct PipelineConstChangedEvent {
    entity: Entity,
}
impl GpuComputeTaskChangeEvent for PipelineConstChangedEvent {
    fn new(entity: Entity) -> Self {
        PipelineConstChangedEvent { entity }
    }
    fn entity(&self) -> Entity {
        self.entity
    }
}
#[derive(Event)]
pub struct MaxOutputLengthChangedEvent {
    entity: Entity,
}
impl GpuComputeTaskChangeEvent for MaxOutputLengthChangedEvent {
    fn new(entity: Entity) -> Self {
        MaxOutputLengthChangedEvent { entity }
    }
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Event)]
pub struct GpuComputeTaskSuccessEvent {
    pub id: u128,
}
