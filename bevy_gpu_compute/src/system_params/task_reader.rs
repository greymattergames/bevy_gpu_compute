use bevy::{ecs::system::SystemParam, prelude::Query};
use bevy_gpu_compute_core::OutputDataBuilderTrait;

use crate::task::task_components::task::BevyGpuComputeTask;

#[derive(SystemParam)]

pub struct GpuTaskReader<'w, 's> {
    tasks: Query<'w, 's, &'static mut BevyGpuComputeTask>,
}

impl<'w, 's> GpuTaskReader<'w, 's> {
    /// the latest result is cleared after this call, you cannot retrieve it a second time
    pub fn latest_results<OutputDataBuilder: OutputDataBuilderTrait>(
        &mut self,
        name: &str,
    ) -> Result<OutputDataBuilder, String> {
        let mut task = self
            .tasks
            .iter_mut()
            .find(|task| task.name() == name)
            .expect("Task not found");
        let result = if let Some(d) = &task.output_data {
            Ok(OutputDataBuilder::from(d))
        } else {
            Err("No output data found".into())
        };
        task.output_data = None;
        result
    }
}
