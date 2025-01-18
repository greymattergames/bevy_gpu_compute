use bevy::{
    app::{App, Plugin, Update},
    prelude::{IntoSystemConfigs, States, in_state},
};

use crate::{
    ram_limit::RamLimit,
    resource::GpuAcceleratedBevy,
    run_ids::GpuAcceleratedBevyRunIds,
    spawn_fallback_camera::{spawn_fallback_camera, spawn_fallback_camera_runif},
    system_sets::compose_task_runner_systems,
    task::{
        events::{
            GpuAcceleratedTaskCreatedEvent, GpuComputeTaskSuccessEvent, InputDataChangeEvent,
            IterSpaceOrOutputSizesChangedEvent,
        },
        setup_tasks::setup_new_tasks,
    },
};

/// state for activating or deactivating the plugin
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GpuAcceleratedBevyState {
    Running,
    Stopped,
}
pub struct GpuAcceleratedBevyPlugin {
    with_default_schedule: bool,
}

impl Plugin for GpuAcceleratedBevyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuAcceleratedBevy>()
            .init_resource::<GpuAcceleratedBevyRunIds>()
            .init_resource::<RamLimit>();
        if self.with_default_schedule {
            let run_tasks_system_set = compose_task_runner_systems();

            app.add_systems(
                Update,
                (
                    spawn_fallback_camera.run_if(spawn_fallback_camera_runif),
                    setup_new_tasks,
                    run_tasks_system_set,
                )
                    .chain()
                    .run_if(in_state(GpuAcceleratedBevyState::Running)),
            );
        } else {
            app.add_systems(
                Update,
                spawn_fallback_camera
                    .run_if(spawn_fallback_camera_runif)
                    .run_if(in_state(GpuAcceleratedBevyState::Running)),
            );
        }
        app.add_event::<GpuComputeTaskSuccessEvent>()
            .add_event::<InputDataChangeEvent>()
            .add_event::<IterSpaceOrOutputSizesChangedEvent>()
            .add_event::<GpuAcceleratedTaskCreatedEvent>();
    }
}

impl Default for GpuAcceleratedBevyPlugin {
    fn default() -> Self {
        GpuAcceleratedBevyPlugin {
            with_default_schedule: true,
        }
    }
}
impl GpuAcceleratedBevyPlugin {
    pub fn no_default_schedule() -> Self {
        GpuAcceleratedBevyPlugin {
            with_default_schedule: false,
        }
    }
}
