use bevy::{
    log,
    prelude::{Query, Res},
};

use crate::ram_limit::RamLimit;

use super::task_components::task_max_output_bytes::TaskMaxOutputBytes;

pub fn verify_have_enough_memory(tasks: Query<&TaskMaxOutputBytes>, ram_limit: Res<RamLimit>) {
    let total_bytes = tasks
        .iter()
        .fold(0, |sum, output_bytes| sum + output_bytes.get());
    let available_memory = ram_limit.total_mem;
    if total_bytes as f32 > available_memory as f32 * 0.9 {
        log::error!(
            "Not enough memory to store all outputs, either reduce the number of entities or allow more potential collision misses by lowering the max_detectable_collisions_scale"
        );
        log::info!(
            "Available memory: {} GB",
            available_memory as f32 / 1024.0 / 1024.0 / 1024.0
        );
        log::info!(
            "Max Output size: {} GB",
            total_bytes as f32 / 1024.0 / 1024.0 / 1024.0
        );
        panic!("Not enough memory to store all outputs");
    }
}
