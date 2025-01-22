use bevy::{
    app::{App, Plugin, Startup, Update},
    prelude::{AppExtStates, IntoSystemConfigs, States, in_state},
};

use crate::{
    ram_limit::RamLimit,
    resource::BevyGpuCompute,
    run_ids::BevyGpuComputeRunIds,
    spawn_fallback_camera::{spawn_fallback_camera, spawn_fallback_camera_runif},
    system_sets::compose_task_runner_systems,
    task::{
        events::{
            ConfigInputDataChangeEvent, GpuAcceleratedTaskCreatedEvent, GpuComputeTaskSuccessEvent,
            InputDataChangeEvent, IterSpaceOrOutputSizesChangedEvent,
        },
        setup_tasks::setup_new_tasks,
    },
};

/// state for activating or deactivating the plugin
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BevyGpuComputeState {
    Running,
    Stopped,
}
impl Default for BevyGpuComputeState {
    fn default() -> Self {
        BevyGpuComputeState::Running
    }
}

pub struct BevyGpuComputePlugin {
    with_default_schedule: bool,
}

impl Plugin for BevyGpuComputePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyGpuCompute>()
            .init_resource::<BevyGpuComputeRunIds>()
            .init_resource::<RamLimit>()
            .init_state::<BevyGpuComputeState>()
            .add_systems(Update, (starting_gpu_tasks, finished_gpu_tasks));
        if self.with_default_schedule {
            let run_tasks_system_set = compose_task_runner_systems();

            app.add_systems(Startup, spawn_fallback_camera).add_systems(
                Update,
                (
                    spawn_fallback_camera.run_if(spawn_fallback_camera_runif),
                    setup_new_tasks,
                    run_tasks_system_set,
                )
                    .chain()
                    .before(finished_gpu_tasks)
                    .after(starting_gpu_tasks)
                    .run_if(in_state(BevyGpuComputeState::Running)),
            );
        } else {
            app.add_systems(
                Update,
                spawn_fallback_camera
                    .run_if(spawn_fallback_camera_runif)
                    .before(finished_gpu_tasks)
                    .after(starting_gpu_tasks)
                    .run_if(in_state(BevyGpuComputeState::Running)),
            );
        }
        app.add_event::<GpuComputeTaskSuccessEvent>();
    }
}

impl Default for BevyGpuComputePlugin {
    fn default() -> Self {
        BevyGpuComputePlugin {
            with_default_schedule: true,
        }
    }
}
impl BevyGpuComputePlugin {
    pub fn no_default_schedule() -> Self {
        BevyGpuComputePlugin {
            with_default_schedule: false,
        }
    }
}

/// used to assist the user with system ordering
pub fn starting_gpu_tasks() {}
/// used to assist the user with system ordering
pub fn finished_gpu_tasks() {}
