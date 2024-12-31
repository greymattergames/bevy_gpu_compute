use bevy::prelude::{Entity, Event};

use crate::code::manager_resource::GpuComputeBevyTaskType;

use super::outputs::{output_data::OutputData, output_spec::OutputVectorTypesSpec};

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
pub struct MaxOutputVectorLengthsChangedEvent {
    entity: Entity,
}
impl GpuComputeTaskChangeEvent for MaxOutputVectorLengthsChangedEvent {
    fn new(entity: Entity) -> Self {
        MaxOutputVectorLengthsChangedEvent { entity }
    }
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Event)]
pub struct IterationSpaceChangedEvent {
    entity: Entity,
}
impl GpuComputeTaskChangeEvent for IterationSpaceChangedEvent {
    fn new(entity: Entity) -> Self {
        IterationSpaceChangedEvent { entity }
    }
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Event)]
pub struct WgslCodeChangedEvent {
    entity: Entity,
}
impl GpuComputeTaskChangeEvent for WgslCodeChangedEvent {
    fn new(entity: Entity) -> Self {
        WgslCodeChangedEvent { entity }
    }
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Event)]
pub struct GpuComputeTaskSuccessEvent {
    pub id: u128,
    // pub data: OutputData<T>,
}
