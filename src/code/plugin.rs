use bevy::{
    app::{App, Plugin, Update},
    prelude::{IntoSystemConfigs, SystemSet},
};

use super::compute_task::{
    buffers::{
        create_input_buffers::{self, create_input_buffers},
        create_output_buffers::create_output_buffers,
    },
    dispatch::{
        create_bind_group::create_bind_groups,
        dispatch_to_gpu::{self, dispatch_to_gpu},
    },
    events::{
        GpuComputeTaskChangeEvent, GpuComputeTaskSuccessEvent, InputDataChangeEvent,
        IterationSpaceChangedEvent, MaxOutputVectorLengthsChangedEvent, WgslCodeChangedEvent,
    },
    iteration_space_dependent_components::{
        pipeline::update::update_pipeline, update_wgsl_params::update_wgsl_params,
    },
    outputs::{
        get_results_counts::get_results_counts_from_gpu,
        read_results_from_gpu::read_results_from_gpu,
    },
};
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct GpuAcceleratedBevyRunTaskSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct GpuAcceleratedBevyRespondToTaskMutSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct GpuAcceleratedBevyReadFromGpuSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct GpuAcceleratedBevyRespondToInputsMutSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct GpuAcceleratedBevyDispatchSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct GpuAcceleratedBevyReadSet;

pub struct GpuAcceleratedBevyPlugin {}

impl Plugin for GpuAcceleratedBevyPlugin {
    fn build(&self, app: &mut App) {
        let respond_to_task_alteration =
            (update_wgsl_params, update_pipeline).in_set(GpuAcceleratedBevyRespondToTaskMutSet);
        let respond_to_new_inputs = (create_input_buffers, create_output_buffers)
            .in_set(GpuAcceleratedBevyRespondToInputsMutSet);
        let dispatch = (create_bind_groups, dispatch_to_gpu)
            .chain()
            .in_set(GpuAcceleratedBevyDispatchSet);
        let read = (output_readers, read_results_from_gpu)
            .chain()
            .in_set(GpuAcceleratedBevyReadSet);
        let set = (
            respond_to_task_alteration,
            respond_to_new_inputs,
            dispatch,
            read,
        )
            .chain()
            .in_set(GpuAcceleratedBevyRunTaskSet);

        // app.configure_sets(Update, (

        //         .run_if(audio_enabled),
        //     MyGameplaySet::Player
        //         .after(MyInputSet)
        //         .run_if(player_is_alive),
        //     MyGameplaySet::Enemies
        //         .run_if(enemies_present),
        //     MyInputKindSet::Touch
        //         .run_if(touchscreen_enabled),
        //     MyInputKindSet::Mouse
        //         .run_if(mouse_enabled),
        //     MyInputKindSet::Gamepad
        //         .run_if(gamepad_connected),
        // ));
        app.add_systems(Update, set);
        app.add_event::<GpuComputeTaskSuccessEvent>();
        app.add_event::<InputDataChangeEvent>();
        app.add_event::<MaxOutputVectorLengthsChangedEvent>();
        app.add_event::<IterationSpaceChangedEvent>();
        app.add_event::<WgslCodeChangedEvent>();

        // Schedule::new(BatchedCollisionDetectionSchedule);
        // batched_collision_detection_schedule.add_systems(
        // (
        //
        // )
        //     .chain(),
        // );
        // app.add_schedule(batched_collision_detection_schedule)
        // .add_systems(Startup, setup_single_batch_resources);
        // }
        // }
    }
}
