use bevy::{
    log,
    prelude::{Commands, Component, DespawnRecursiveExt, Entity, Event, Query, ResMut},
};

use crate::run_ids::GpuAcceleratedBevyRunIds;

use super::{
    events::{
        GpuComputeTaskChangeEvent, InputDataChangeEvent,
        IterationSpaceOrMaxOutVecLengthChangedEvent, WgslCodeChangedEvent,
    },
    inputs::{
        input_data::InputData, input_vector_types_spec::InputVectorTypesSpec,
        type_erased_input_data::TypeErasedInputData,
    },
    iteration_space::iteration_space::IterationSpace,
    outputs::definitions::{
        max_output_vector_lengths::MaxOutputVectorLengths, output_data::OutputData,
        output_vector_types_spec::OutputVectorTypesSpec,
        type_erased_output_data::TypeErasedOutputData,
    },
    task_components::task_run_id::TaskRunId,
    wgsl_code::WgslCode,
};
#[derive(Clone, Debug)]
pub struct TaskCommands {
    entity: Entity,
}
impl TaskCommands {
    pub fn new(entity: Entity) -> Self {
        TaskCommands { entity }
    }

    pub fn delete(&self, commands: &mut Commands) {
        commands.entity(self.entity).despawn_recursive();
    }

    pub fn set_iteration_space(
        &self,
        commands: &mut Commands,
        new_iteration_space: IterationSpace,
    ) {
        self.alter_task::<IterationSpaceOrMaxOutVecLengthChangedEvent, _>(
            commands,
            new_iteration_space,
        );
    }
    pub fn set_max_output_vector_lengths(
        &self,
        commands: &mut Commands,
        new_max_output_vector_lengths: MaxOutputVectorLengths,
    ) {
        self.alter_task::<IterationSpaceOrMaxOutVecLengthChangedEvent, _>(
            commands,
            new_max_output_vector_lengths,
        );
    }
    pub fn set_wgsl_code(&self, commands: &mut Commands, new_wgsl_code: WgslCode) {
        self.alter_task::<WgslCodeChangedEvent, _>(commands, new_wgsl_code);
    }
    fn alter_task<E: Event + GpuComputeTaskChangeEvent, T: Component>(
        &self,
        commands: &mut Commands,
        new_component: T,
    ) {
        let mut entity_commands = commands.entity(self.entity);
        entity_commands.insert(new_component);
        commands.send_event(E::new(self.entity));
    }

    /// registers the input data to run in the next round, returns a unique id to identify the run
    pub fn run<I: InputVectorTypesSpec + 'static + Send + Sync>(
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

    pub fn result<O: OutputVectorTypesSpec>(
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
