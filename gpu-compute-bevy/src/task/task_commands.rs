use bevy::{
    log,
    prelude::{Commands, Component, DespawnRecursiveExt, Entity, Event, Query, ResMut},
};
use shared::misc_types::{InputVectorTypesSpec, OutputVectorTypesSpec, TypesSpec};

use crate::run_ids::GpuAcceleratedBevyRunIds;

use super::{
    events::{
        GpuComputeTaskChangeEvent, InputDataChangeEvent,
        IterationSpaceOrMaxOutVecLengthChangedEvent, WgslCodeChangedEvent,
    },
    inputs::{input_data::InputData, type_erased_input_data::TypeErasedInputData},
    outputs::definitions::{
        output_data::OutputData,
        output_vector_metadata_spec::{self, OutputVectorsMetadataSpec},
        type_erased_output_data::TypeErasedOutputData,
    },
    task_components::task_run_id::TaskRunId,
    task_specification::{
        iteration_space::IterationSpace, task_specification::ComputeTaskSpecification,
    },
    wgsl_code::WgslCode,
};
#[derive(Clone, Debug)]
pub struct TaskCommands {
    pub entity: Entity,
}
impl TaskCommands {
    pub fn new(entity: Entity) -> Self {
        TaskCommands { entity }
    }
    pub fn delete(&self, commands: &mut Commands) {
        commands.entity(self.entity).despawn_recursive();
    }

    /// registers the input data to run in the next round, returns a unique id to identify the run
    pub fn run<I: TypesSpec + 'static + Send + Sync>(
        &self,
        commands: &mut Commands,
        inputs: InputData<I>,
        mut task_run_ids: ResMut<GpuAcceleratedBevyRunIds>,
    ) -> u128 {
        let mut entity_commands = commands.entity(self.entity);
        let id = task_run_ids.get_next();
        log::info!("run id: {}", id);
        // log::info!("inputs: {:?}", inputs);
        entity_commands.insert(TypeErasedInputData::new::<I>(inputs));
        entity_commands.insert(TaskRunId(id));
        commands.send_event(InputDataChangeEvent::new(self.entity));
        id
    }

    pub fn result<O: TypesSpec>(
        &self,
        run_id: u128,
        out_datas: &Query<(&TaskRunId, &TypeErasedOutputData)>,
    ) -> Option<OutputData<O>> {
        log::info!("looking for output data for run id: {}", run_id);
        for (task_run_id, type_erased_data) in out_datas.iter() {
            if task_run_id.0 == run_id {
                log::info!("found output data for run id: {}", run_id);
                return type_erased_data.clone().into_typed::<O>().ok();
            }
        }
        None
    }
}
