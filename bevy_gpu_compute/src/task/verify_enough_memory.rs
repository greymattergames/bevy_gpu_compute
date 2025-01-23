use bevy::{
    log,
    prelude::{Query, Res},
};

use crate::ram_limit::RamLimit;

use super::{
    task_components::task::BevyGpuComputeTask,
    task_specification::task_specification::ComputeTaskSpecification,
};

pub fn verify_have_enough_memory(tasks: Vec<&BevyGpuComputeTask>, ram_limit: &RamLimit) {
    let total_bytes = tasks.iter().fold(0, |sum, task_spec| {
        sum + task_spec.spec.task_max_output_bytes().get()
    });
    let available_memory = ram_limit.total_mem;
    if total_bytes as f32 > available_memory as f32 * 0.9 {
        log::error!("Not enough memory to store all gpu compute task outputs");
        log::info!(
            "Available memory: {} GB",
            available_memory as f32 / 1024.0 / 1024.0 / 1024.0
        );
        log::info!(
            "Max Output size: {} GB",
            total_bytes as f32 / 1024.0 / 1024.0 / 1024.0
        );
        panic!("Not enough memory to store all gpu compute task outputs");
    }
}
