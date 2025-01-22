use bevy::{
    log,
    prelude::{Commands, DespawnRecursiveExt, Entity, Query, ResMut},
};
use bevy_gpu_compute_core::TypesSpec;

use crate::{
    prelude::{ComputeTaskSpecification, IterationSpace, MaxOutputLengths},
    run_ids::BevyGpuComputeRunIds,
    task::inputs::array_type::input_data::InputDataTrait,
};

use super::{
    events::{ConfigInputDataChangeEvent, InputDataChangeEvent},
    inputs::{
        array_type::{input_data::InputData, type_erased_input_data::TypeErasedInputData},
        config_type::{
            config_data::ConfigInputData, type_erased_config_input_data::TypeErasedConfigInputData,
        },
    },
    outputs::definitions::{
        output_data::OutputData, type_erased_output_data::TypeErasedOutputData,
    },
    task_components::task_run_id::TaskRunId,
    task_specification::{self, input_array_lengths::ComputeTaskInputArrayLengths},
};
pub struct TaskCommands<'w, 's> {
    entity: Entity,
    commands: &'w mut Commands<'w, 's>,
    spec: &'w mut ComputeTaskSpecification,
}
impl<'w, 's> TaskCommands<'w, 's> {
    pub fn new(
        entity: Entity,
        commands: &'w mut Commands<'w, 's>,
        spec: &'w mut ComputeTaskSpecification,
    ) -> Self {
        TaskCommands {
            entity,
            commands,
            spec,
        }
    }
    pub fn delete(&mut self) {
        self.commands.entity(self.entity).despawn_recursive();
    }
    pub fn mutate(
        &self,
        new_iteration_space: Option<IterationSpace>,
        new_max_output_array_lengths: Option<MaxOutputLengths>,
    ) {
        self.spec.mutate(
            &mut commands,
            self.entity,
            new_iteration_space,
            new_max_output_array_lengths,
            new_input_array_lengths,
        );
    }
    pub fn set_config_inputs<I: TypesSpec + 'static + Send + Sync>(
        &self,
        inputs: ConfigInputData<I>,
    ) {
        let mut entity_commands = commands.entity(self.entity);
        let event = ConfigInputDataChangeEvent::new(self.entity);
        entity_commands.insert(TypeErasedConfigInputData::new::<I>(inputs));
        commands.send_event(event);
    }

    /// registers the input data to run in the next round, returns a unique id to identify the run
    pub fn run<I: TypesSpec + 'static + Send + Sync>(
        &self,
        commands: &mut Commands,
        inputs: InputData<I>,
        mut task_run_ids: ResMut<BevyGpuComputeRunIds>,
    ) -> u128 {
        let mut entity_commands = commands.entity(self.entity);
        let id = task_run_ids.get_next();
        let event = InputDataChangeEvent::new(self.entity, inputs.lengths());
        log::info!("run id: {}", id);
        // log::info!("inputs: {:?}", inputs);
        entity_commands.insert(TypeErasedInputData::new::<I>(inputs));
        entity_commands.insert(TaskRunId(id));
        commands.send_event(event);
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
