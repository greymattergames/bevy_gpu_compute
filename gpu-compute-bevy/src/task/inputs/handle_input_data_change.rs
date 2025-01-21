use bevy::{
    ecs::batching::BatchingStrategy,
    log,
    prelude::{Commands, EventReader, Query},
};

use crate::task::{
    events::InputDataChangeEvent,
    task_specification::{
        input_array_lengths::ComputeTaskInputArrayLengths,
        task_specification::ComputeTaskSpecification,
    },
};

pub fn handle_input_data_change(
    mut commands: Commands,
    mut tasks: Query<&mut ComputeTaskSpecification>,
    mut event_reader: EventReader<InputDataChangeEvent>,
) {
    for (ev, _) in event_reader
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        log::info!("handle_input_data_change");
        let entity = ev.entity();
        let lengths_unnamed = ev.lengths;
        let mut task = tasks.get_mut(entity);
        if let Ok(t) = task.as_mut() {
            t.mutate(
                &mut commands,
                entity,
                None,
                None,
                Some(ComputeTaskInputArrayLengths {
                    by_index: lengths_unnamed,
                }),
            );
        }
    }
}
